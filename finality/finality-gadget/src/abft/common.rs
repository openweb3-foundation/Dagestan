use std::{sync::Arc, time::Duration};

use crate::{NodeIndex, SessionId, UnitCreationDelay};

const MAX_ROUNDS: u16 = 7000;

fn exponential_slowdown(
    t: usize,
    base_delay: f64,
    start_exp_delay: usize,
    exp_base: f64,
) -> Duration {
    // This gives:
    // base_delay, for t <= start_exp_delay,
    // base_delay * exp_base^(t - start_exp_delay), for t > start_exp_delay.
    let delay = if t < start_exp_delay {
        base_delay
    } else {
        let power = t - start_exp_delay;
        base_delay * exp_base.powf(power as f64)
    };
    let delay = delay.round() as u64;
    // the above will make it u64::MAX if it exceeds u64
    Duration::from_millis(delay)
}

pub type DelaySchedule = Arc<dyn Fn(usize) -> Duration + Sync + Send + 'static>;
pub type RecipientCountSchedule = Arc<dyn Fn(usize) -> usize + Sync + Send + 'static>;

pub fn unit_creation_delay_fn(unit_creation_delay: UnitCreationDelay) -> DelaySchedule {
    Arc::new(move |t| match t {
        0 => Duration::from_millis(2000),
        _ => exponential_slowdown(t, unit_creation_delay.0 as f64, 5000, 1.005),
    })
}

pub struct DelayConfig {
    pub tick_interval: Duration,
    pub requests_interval: Duration,
    pub unit_rebroadcast_interval_min: Duration,
    pub unit_rebroadcast_interval_max: Duration,
    pub unit_creation_delay: DelaySchedule,
    pub coord_request_delay: DelaySchedule,
    pub coord_request_recipients: RecipientCountSchedule,
    pub parent_request_delay: DelaySchedule,
    pub parent_request_recipients: RecipientCountSchedule,
    pub newest_request_delay: DelaySchedule,
}

pub struct DagestanConfig {
    delay_config: DelayConfig,
    n_members: usize,
    node_id: NodeIndex,
    session_id: SessionId,
}

impl DagestanConfig {
    pub fn new(
        delay_config: DelayConfig,
        n_members: usize,
        node_id: NodeIndex,
        session_id: SessionId,
    ) -> DagestanConfig {
        DagestanConfig {
            delay_config,
            n_members,
            node_id,
            session_id,
        }
    }
}

impl From<DelayConfig> for legacy_aleph_bft::DelayConfig {
    fn from(cfg: DelayConfig) -> Self {
        Self {
            tick_interval: cfg.tick_interval,
            requests_interval: cfg.requests_interval,
            unit_rebroadcast_interval_max: cfg.unit_rebroadcast_interval_max,
            unit_rebroadcast_interval_min: cfg.unit_rebroadcast_interval_min,
            unit_creation_delay: cfg.unit_creation_delay,
        }
    }
}

impl From<DelayConfig> for current_aleph_bft::DelayConfig {
    fn from(cfg: DelayConfig) -> Self {
        Self {
            tick_interval: cfg.tick_interval,
            unit_rebroadcast_interval_max: cfg.unit_rebroadcast_interval_max,
            unit_rebroadcast_interval_min: cfg.unit_rebroadcast_interval_min,
            unit_creation_delay: cfg.unit_creation_delay,
            coord_request_delay: cfg.coord_request_delay,
            coord_request_recipients: cfg.coord_request_recipients,
            parent_request_delay: cfg.parent_request_delay,
            parent_request_recipients: cfg.parent_request_recipients,
            newest_request_delay: cfg.newest_request_delay,
        }
    }
}

impl From<DagestanConfig> for current_aleph_bft::Config {
    fn from(cfg: DagestanConfig) -> Self {
        let mut dagestan_config = current_aleph_bft::default_config(
            cfg.n_members.into(),
            cfg.node_id.into(),
            cfg.session_id.0 as u64,
        );
        dagestan_config.max_round = MAX_ROUNDS;
        dagestan_config.delay_config = cfg.delay_config.into();

        dagestan_config
    }
}

impl From<DagestanConfig> for legacy_aleph_bft::Config {
    fn from(cfg: DagestanConfig) -> Self {
        let mut dagestan_config = legacy_aleph_bft::default_config(
            cfg.n_members.into(),
            cfg.node_id.into(),
            cfg.session_id.0 as u64,
        );
        dagestan_config.max_round = MAX_ROUNDS;
        dagestan_config.delay_config = cfg.delay_config.into();

        dagestan_config
    }
}
