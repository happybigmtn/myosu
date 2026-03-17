//! Config trait definition for pallet-game-solver.
//!
//! This module defines the Config trait WITHOUT the drand/crowdloan supertraits
//! that subtensor requires. This is the foundational change that unblocks
//! the entire chain fork (CF-07).
//!
//! Subtensor's Config trait includes:
//! ```ignore
//! pub trait Config:
//!     frame_system::Config
//!     + pallet_drand::Config          // STRIPPED - not needed for game-solving
//!     + pallet_crowdloan::Config      // STRIPPED - not needed for game-solving
//!     + ...
//! {}
//! ```
//!
//! Myosu's Config trait includes only what is needed for game-solving consensus:
//! - frame_system::Config (required by all pallets)
//! - Currency (for token operations)
//! - SwapHandler + SwapEngine (via NoOpSwap stub - single token model)
//! - ProxyInterface, CommitmentsInterface, AuthorshipProvider (via stubs)

/// Define the Config trait for pallet-game-solver.
///
/// This macro expands to the full trait definition with all required bounds
/// but WITHOUT drand/crowdloan dependencies.
#[macro_export]
macro_rules! config_trait {
    () => {
        /// Configuration trait for the game-solving pallet.
        ///
        /// This trait defines the types and constants needed for the myosu
        /// game-solving subnet protocol. It is intentionally minimal and
        /// strips out subtensor-specific dependencies like drand and crowdloan.
        pub trait Config: frame_system::Config {
            /// The overarching event type.
            type RuntimeEvent: From<Event<Self>>
                + IsType<<Self as frame_system::Config>::RuntimeEvent>;

            /// The currency mechanism for token operations.
            type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

            /// Balance type alias for consistency.
            type Balance: BalanceType;

            /// Swap handler for the single-token model.
            /// In subtensor this is a full AMM; in myosu it's a no-op identity stub.
            type SwapHandler: SwapHandler<Balance = Self::Balance>;

            /// Swap engine for TAO->Alpha conversions (identity in single-token model).
            type SwapEngineAlpha: SwapEngine<GetAlphaForTao<Self>, Balance = Self::Balance>;

            /// Swap engine for Alpha->TAO conversions (identity in single-token model).
            type SwapEngineTao: SwapEngine<GetTaoForAlpha<Self>, Balance = Self::Balance>;

            /// Proxy interface for account delegation (no-op stub in stage 0).
            type ProxyInterface: ProxyInterface<Self::AccountId>;

            /// Commitments interface for arbitrary data commitments (no-op stub).
            type CommitmentsInterface: CommitmentsInterface<Self::AccountId>;

            /// Authorship provider for block author tracking.
            type AuthorshipProvider: AuthorshipProvider<Self::AccountId>;

            /// Origin for privileged operations (typically Root).
            type SenateMembers: EnsureOrigin<Self::RuntimeOrigin>;

            /// Initial tempo (blocks between epochs) for new subnets.
            #[pallet::constant]
            type InitialTempo: Get<u16>;

            /// Initial kappa (consensus clipping threshold as u16 per-mille).
            #[pallet::constant]
            type InitialKappa: Get<u16>;

            /// Initial bonds moving average (alpha parameter as u16 per-mille).
            #[pallet::constant]
            type InitialBondsMovingAverage: Get<u16>;

            /// Initial bonds penalty (beta parameter as u16 per-mille).
            #[pallet::constant]
            type InitialBondsPenalty: Get<u16>;

            /// Initial immunity period (blocks new neurons are protected from pruning).
            #[pallet::constant]
            type InitialImmunityPeriod: Get<u16>;

            /// Initial activity cutoff (blocks before inactive validators are penalized).
            #[pallet::constant]
            type InitialActivityCutoff: Get<u16>;

            /// Initial maximum allowed validators per subnet.
            #[pallet::constant]
            type InitialMaxAllowedValidators: Get<u16>;

            /// Initial minimum allowed weights per validator.
            #[pallet::constant]
            type InitialMinAllowedWeights: Get<u16>;

            /// Initial maximum allowed UIDs (neurons) per subnet.
            #[pallet::constant]
            type InitialMaxUids: Get<u16>;

            /// Initial burn cost for neuron registration.
            #[pallet::constant]
            type InitialBurn: Get<Self::Balance>;

            /// Initial difficulty for neuron registration.
            #[pallet::constant]
            type InitialDifficulty: Get<Self::Balance>;

            /// Initial stake pruning denominator (for low-stake removal).
            #[pallet::constant]
            type InitialStakePruningDenominator: Get<u16>;

            /// Minimum stake required for validator permits.
            #[pallet::constant]
            type MinStakePerWeight: Get<Self::Balance>;

            /// Maximum weight value (u16::MAX for 16-bit weights).
            #[pallet::constant]
            type MaxWeight: Get<u16>;

            /// Number of blocks in a reveal period for commit-reveal.
            #[pallet::constant]
            type RevealPeriod: Get<u64>;

            /// Initial subnet limit (maximum number of subnets).
            #[pallet::constant]
            type InitialSubnetLimit: Get<u16>;

            /// Cost to create a new subnet.
            #[pallet::constant]
            type SubnetCreationCost: Get<Self::Balance>;

            /// Emission per block in RAO (smallest unit).
            #[pallet::constant]
            type BlockEmission: Get<u64>;
        }
    };
}

/// Re-export commonly used types for Config implementations.
pub mod types {
    pub use frame_support::pallet_prelude::*;
    pub use frame_support::traits::{Currency, ReservableCurrency};
    pub use sp_runtime::traits::AtLeast32BitUnsigned;

    /// Trait alias for balance types used in the pallet.
    pub trait BalanceType:
        Copy + Clone + Default + core::fmt::Debug + AtLeast32BitUnsigned + parity_scale_codec::Codec
    {
    }

    impl<T> BalanceType for T where
        T: Copy + Clone + Default + core::fmt::Debug + AtLeast32BitUnsigned + parity_scale_codec::Codec
    {
    }
}

// Re-export the types at module level for convenience
pub use types::*;

/// Placeholder trait definitions that will be implemented in other modules.
/// These are declared here to satisfy the Config trait bounds.
pub trait SwapHandler {
    type Balance;
}

pub trait SwapEngine<Direction> {
    type Balance;
}

pub struct GetAlphaForTao<T>(core::marker::PhantomData<T>);
pub struct GetTaoForAlpha<T>(core::marker::PhantomData<T>);

pub trait ProxyInterface<AccountId> {
    fn exists(delegate: &AccountId) -> bool;
    fn proxied(who: &AccountId) -> Option<AccountId>;
    fn real(who: AccountId) -> AccountId;
    fn is_pure(who: &AccountId) -> bool;
}

pub trait CommitmentsInterface<AccountId> {
    fn set_commitment(who: &AccountId, data: &[u8]) -> Result<(), ()>;
    fn get_commitment(who: &AccountId) -> Option<alloc::vec::Vec<u8>>;
    fn rate_limit() -> u64;
}

pub trait AuthorshipProvider<AccountId> {
    fn author() -> Option<AccountId>;
    fn uncles() -> alloc::vec::Vec<AccountId>;
    fn set_author(author: &AccountId);
}
