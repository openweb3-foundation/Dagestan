#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]
#![cfg_attr(not(feature = "std"), no_std)]
use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::crypto::KeyTypeId;
pub use sp_runtime::{
    generic::Header as GenericHeader,
    traits::{BlakeTwo256, ConstU32, Header as HeaderT},
    BoundedVec, ConsensusEngineId, Perbill,
};
pub use sp_staking::{EraIndex, SessionIndex};
use sp_std::vec::Vec;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"alp0");

// Same as GRANDPA_ENGINE_ID because as of right now substrate sends only
// grandpa justifications over the network.
// TODO: change this once https://github.com/paritytech/substrate/issues/8172 will be resolved.
pub const ALEPH_ENGINE_ID: ConsensusEngineId = *b"FRNK";

mod app {
    use sp_application_crypto::{app_crypto, ed25519};
    app_crypto!(ed25519, crate::KEY_TYPE);
}

sp_application_crypto::with_pair! {
    pub type AuthorityPair = app::Pair;
}
pub type AuthoritySignature = app::Signature;
pub type AuthorityId = app::Public;

pub type BlockNumber = u32;
pub type SessionCount = u32;
pub type BlockCount = u32;


/// Openness of the process of the elections
#[derive(Decode, Encode, TypeInfo, Debug, Clone, PartialEq, Eq)]
pub enum ElectionOpenness {
    Permissioned,
    Permissionless,
}

/// Represent desirable size of a committee in a session
#[derive(Decode, Encode, TypeInfo, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CommitteeSeats {
    /// Size of reserved validators in a session
    pub reserved_seats: u32,
    /// Size of non reserved valiadtors in a session
    pub non_reserved_seats: u32,
}

impl CommitteeSeats {
    pub fn size(&self) -> u32 {
        self.reserved_seats.saturating_add(self.non_reserved_seats)
    }
}

impl Default for CommitteeSeats {
    fn default() -> Self {
        CommitteeSeats {
            reserved_seats: DEFAULT_COMMITTEE_SIZE,
            non_reserved_seats: 0,
        }
    }
}

/// Configurable parameters for ban validator mechanism
#[derive(Decode, Encode, TypeInfo, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BanConfig {
    /// performance ratio threshold in a session
    /// calculated as ratio of number of blocks produced to expected number of blocks for a single validator
    pub minimal_expected_performance: Perbill,
    /// how many bad uptime sessions force validator to be removed from the committee
    pub underperformed_session_count_threshold: SessionCount,
    /// underperformed session counter is cleared every subsequent `clean_session_counter_delay` sessions
    pub clean_session_counter_delay: SessionCount,
    /// how many eras a validator is banned for
    pub ban_period: EraIndex,
}

impl Default for BanConfig {
    fn default() -> Self {
        BanConfig {
            minimal_expected_performance: DEFAULT_BAN_MINIMAL_EXPECTED_PERFORMANCE,
            underperformed_session_count_threshold: DEFAULT_BAN_SESSION_COUNT_THRESHOLD,
            clean_session_counter_delay: DEFAULT_CLEAN_SESSION_COUNTER_DELAY,
            ban_period: DEFAULT_BAN_PERIOD,
        }
    }
}

/// Represent any possible reason a validator can be removed from the committee due to
#[derive(PartialEq, Eq, Clone, Encode, Decode, TypeInfo, Debug)]
pub enum BanReason {
    /// Validator has been removed from the committee due to insufficient uptime in a given number
    /// of sessions
    InsufficientUptime(u32),

    /// Any arbitrary reason
    OtherReason(BoundedVec<u8, ConstU32<DEFAULT_BAN_REASON_LENGTH>>),
}

/// Details of why and for how long a validator is removed from the committee
#[derive(PartialEq, Eq, Clone, Encode, Decode, TypeInfo, Debug)]
pub struct BanInfo {
    /// reason for banning a validator
    pub reason: BanReason,
    /// index of the first era when a ban starts
    pub start: EraIndex,
}

/// Represent committee, ie set of nodes that produce and finalize blocks in the session
#[derive(Eq, PartialEq, Decode, Encode, TypeInfo)]
pub struct EraValidators<AccountId> {
    /// Validators that are chosen to be in committee every single session.
    pub reserved: Vec<AccountId>,
    /// Validators that can be banned out from the committee, under the circumstances
    pub non_reserved: Vec<AccountId>,
}

impl<AccountId> Default for EraValidators<AccountId> {
    fn default() -> Self {
        Self {
            reserved: Vec::new(),
            non_reserved: Vec::new(),
        }
    }
}

#[derive(Encode, Decode, PartialEq, Eq, Debug)]
pub enum ApiError {
    DecodeKey,
}

/// All the data needed to verify block finalization justifications.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct SessionAuthorityData {
    authorities: Vec<AuthorityId>,
    emergency_finalizer: Option<AuthorityId>,
}

impl SessionAuthorityData {
    pub fn new(authorities: Vec<AuthorityId>, emergency_finalizer: Option<AuthorityId>) -> Self {
        SessionAuthorityData {
            authorities,
            emergency_finalizer,
        }
    }

    pub fn authorities(&self) -> &Vec<AuthorityId> {
        &self.authorities
    }

    pub fn emergency_finalizer(&self) -> &Option<AuthorityId> {
        &self.emergency_finalizer
    }
}

pub type Version = u32;

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct VersionChange {
    pub version_incoming: Version,
    pub session: SessionIndex,
}

sp_api::decl_runtime_apis! {
    pub trait DagestanSessionApi
    {
        fn next_session_authorities() -> Result<Vec<AuthorityId>, ApiError>;
        fn authorities() -> Vec<AuthorityId>;
        fn next_session_authority_data() -> Result<SessionAuthorityData, ApiError>;
        fn authority_data() -> SessionAuthorityData;
        fn session_period() -> u32;
        fn millisecs_per_block() -> u64;
        fn finality_version() -> Version;
        fn next_session_finality_version() -> Version;
    }
}
