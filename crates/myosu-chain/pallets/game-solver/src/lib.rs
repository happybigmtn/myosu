#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "512"]
// Learn more about FRAME and the core library of Substrate FRAME pallets:
// <https://docs.substrate.io/reference/frame-pallets/>

use frame_support::pallet_prelude::*;
use frame_support::traits::{Currency, OriginTrait};

/// Subnet ID type — used everywhere as a subnet identifier.
pub type NetUid = u16;

/// Balance type — single-token model for Myosu.
pub type Balance = u64;

// ============================
// ==== Module Structure ======
// ============================

/// Epoch processing with game-solving math (includes SafeDiv trait).
pub mod epoch;

/// Config trait and related type definitions.
pub mod macros;

// apparently this is stabilized since rust 1.36
extern crate alloc;

// ============================
// ==== Pallet Definition =====
// ============================

#[allow(dead_code)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::macros::config::{
        AuthorshipProvider, BalanceType, CommitmentsInterface, GetAlphaForTao, GetTaoForAlpha,
        ProxyInterface, SwapEngine, SwapHandler,
    };
    use frame_support::traits::ReservableCurrency;
    use frame_system::pallet_prelude::BlockNumberFor;

    /// Configuration trait for the game-solving pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The currency mechanism for token operations.
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Balance type alias for consistency.
        type Balance: BalanceType;

        /// Swap handler for the single-token model.
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

    /// Origin for the pallet
    pub type PalletsOriginOf<T> =
        <<T as frame_system::Config>::RuntimeOrigin as OriginTrait>::PalletsOrigin;

    /// Call type for the pallet
    pub type CallOf<T> = <T as frame_system::Config>::RuntimeCall;

    /// Tracks version for migrations.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(7);

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    // ========
    // Errors
    // ========
    #[pallet::error]
    pub enum Error<T> {
        /// Generic error for out-of-range parameter value.
        InvalidValue,
    }

    // ========
    // Events
    // ========
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A network was added.
        NetworkAdded(T::AccountId, NetUid),
    }

    // ========
    // Hooks
    // ========
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_block_number: BlockNumberFor<T>) -> Weight {
            Weight::zero()
        }

        fn on_finalize(_block_number: BlockNumberFor<T>) {}

        fn on_runtime_upgrade() -> Weight {
            Weight::zero()
        }
    }

    // ========
    // Genesis
    // ========
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub _phantom: sp_std::marker::PhantomData<T>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                _phantom: sp_std::marker::PhantomData,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {}
    }

    // ========
    // Calls
    // ========
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // Dispatchable stubs — subtensor-specific calls stripped for Myosu genesis.
    }
}
