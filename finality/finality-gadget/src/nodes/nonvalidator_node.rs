use log::{debug, error};
use sc_client_api::Backend;
use sc_network_common::ExHashT;
use sp_consensus::SelectChain;
use sp_runtime::traits::Block;

use crate::{
    nodes::{setup_justification_handler, JustificationParams},
    session_map::{AuthorityProviderImpl, FinalityNotificatorImpl, SessionMapUpdater},
    DagestanConfig, BlockchainBackend,
};

pub async fn run_nonvalidator_node<B, H, C, BB, BE, SC>(dagestan_config: DagestanConfig<B, H, C, SC, BB>)
where
    B: Block,
    H: ExHashT,
    C: crate::ClientForDagestan<B, BE> + Send + Sync + 'static,
    C::Api: dagestan_primitives::DagestanSessionApi<B>,
    BE: Backend<B> + 'static,
    BB: BlockchainBackend<B> + Send + 'static,
    SC: SelectChain<B> + 'static,
{
    let DagestanConfig {
        network,
        client,
        blockchain_backend,
        metrics,
        session_period,
        millisecs_per_block,
        justification_rx,
        spawn_handle,
        ..
    } = dagestan_config;
    let map_updater = SessionMapUpdater::<_, _, B>::new(
        AuthorityProviderImpl::new(client.clone()),
        FinalityNotificatorImpl::new(client.clone()),
    );
    let session_authorities = map_updater.readonly_session_map();
    spawn_handle.spawn("dagestan/updater", None, async move {
        debug!(target: "dagestan-party", "SessionMapUpdater has started.");
        map_updater.run(session_period).await
    });
    let (_, handler_task) = setup_justification_handler(JustificationParams {
        justification_rx,
        network,
        client,
        blockchain_backend,
        metrics,
        session_period,
        millisecs_per_block,
        session_map: session_authorities,
    });

    debug!(target: "dagestan-party", "JustificationHandler has started.");
    handler_task.await;
    error!(target: "dagestan-party", "JustificationHandler finished.");
}
