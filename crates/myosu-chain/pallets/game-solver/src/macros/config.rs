#![allow(clippy::crate_in_macro_def)]

use frame_support::pallet_macros::pallet_section;

pub trait GetCommitments<AccountId> {
    fn get_commitments(
        _netuid: crate::NetUid,
    ) -> sp_std::vec::Vec<(AccountId, sp_std::vec::Vec<u8>)> {
        sp_std::vec::Vec::new()
    }
}

#[pallet_section]
mod config {
    use crate::macros::config::GetCommitments;
    use crate::{CommitmentsInterface, Stage0SwapInterface};
    use subtensor_runtime_common::AuthorshipInfo;

    #[allow(missing_docs)]
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + From<Call<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeCall>
            + From<frame_system::Call<Self>>;

        type SudoRuntimeCall: Parameter
            + UnfilteredDispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + GetDispatchInfo;

        type Currency: fungible::Balanced<Self::AccountId, Balance = u64>
            + fungible::Mutate<Self::AccountId>;

        type Scheduler: ScheduleAnon<
                BlockNumberFor<Self>,
                LocalCallOf<Self>,
                PalletsOriginOf<Self>,
                Hasher = Self::Hashing,
            >;

        type Preimages: QueryPreimage<H = Self::Hashing> + StorePreimage;

        type SwapInterface: Stage0SwapInterface<Self>;

        type ProxyInterface: crate::ProxyInterface<Self::AccountId>;
        type GetCommitments: GetCommitments<Self::AccountId>;
        type CommitmentsInterface: CommitmentsInterface;
        type EvmKeyAssociateRateLimit: Get<u64>;
        type AuthorshipProvider: AuthorshipInfo<Self::AccountId>;

        #[pallet::constant]
        type InitialIssuance: Get<u64>;
        #[pallet::constant]
        type InitialMinAllowedWeights: Get<u16>;
        #[pallet::constant]
        type InitialEmissionValue: Get<u16>;
        #[pallet::constant]
        type InitialTempo: Get<u16>;
        #[pallet::constant]
        type InitialDifficulty: Get<u64>;
        #[pallet::constant]
        type InitialMaxDifficulty: Get<u64>;
        #[pallet::constant]
        type InitialMinDifficulty: Get<u64>;
        #[pallet::constant]
        type InitialRAORecycledForRegistration: Get<u64>;
        #[pallet::constant]
        type InitialBurn: Get<u64>;
        #[pallet::constant]
        type InitialMaxBurn: Get<u64>;
        #[pallet::constant]
        type InitialMinBurn: Get<u64>;
        #[pallet::constant]
        type MinBurnUpperBound: Get<subtensor_runtime_common::TaoCurrency>;
        #[pallet::constant]
        type MaxBurnLowerBound: Get<subtensor_runtime_common::TaoCurrency>;
        #[pallet::constant]
        type InitialAdjustmentInterval: Get<u16>;
        #[pallet::constant]
        type InitialBondsMovingAverage: Get<u64>;
        #[pallet::constant]
        type InitialBondsPenalty: Get<u16>;
        #[pallet::constant]
        type InitialBondsResetOn: Get<bool>;
        #[pallet::constant]
        type InitialTargetRegistrationsPerInterval: Get<u16>;
        #[pallet::constant]
        type InitialRho: Get<u16>;
        #[pallet::constant]
        type InitialAlphaSigmoidSteepness: Get<i16>;
        #[pallet::constant]
        type InitialKappa: Get<u16>;
        #[pallet::constant]
        type InitialMinAllowedUids: Get<u16>;
        #[pallet::constant]
        type InitialMaxAllowedUids: Get<u16>;
        #[pallet::constant]
        type InitialValidatorPruneLen: Get<u64>;
        #[pallet::constant]
        type InitialScalingLawPower: Get<u16>;
        #[pallet::constant]
        type InitialImmunityPeriod: Get<u16>;
        #[pallet::constant]
        type InitialActivityCutoff: Get<u16>;
        #[pallet::constant]
        type InitialMaxRegistrationsPerBlock: Get<u16>;
        #[pallet::constant]
        type InitialPruningScore: Get<u16>;
        #[pallet::constant]
        type InitialMaxAllowedValidators: Get<u16>;
        #[pallet::constant]
        type InitialDefaultDelegateTake: Get<u16>;
        #[pallet::constant]
        type InitialMinDelegateTake: Get<u16>;
        #[pallet::constant]
        type InitialDefaultChildKeyTake: Get<u16>;
        #[pallet::constant]
        type InitialMinChildKeyTake: Get<u16>;
        #[pallet::constant]
        type InitialMaxChildKeyTake: Get<u16>;
        #[pallet::constant]
        type InitialWeightsVersionKey: Get<u64>;
        #[pallet::constant]
        type InitialServingRateLimit: Get<u64>;
        #[pallet::constant]
        type InitialTxRateLimit: Get<u64>;
        #[pallet::constant]
        type InitialTxDelegateTakeRateLimit: Get<u64>;
        #[pallet::constant]
        type InitialTxChildKeyTakeRateLimit: Get<u64>;
        #[pallet::constant]
        type InitialAdjustmentAlpha: Get<u64>;
        #[pallet::constant]
        type InitialNetworkImmunityPeriod: Get<u64>;
        #[pallet::constant]
        type InitialNetworkMinLockCost: Get<u64>;
        #[pallet::constant]
        type InitialSubnetOwnerCut: Get<u16>;
        #[pallet::constant]
        type InitialNetworkLockReductionInterval: Get<u64>;
        #[pallet::constant]
        type InitialNetworkRateLimit: Get<u64>;
        #[pallet::constant]
        type KeySwapCost: Get<u64>;
        #[pallet::constant]
        type AlphaHigh: Get<u16>;
        #[pallet::constant]
        type AlphaLow: Get<u16>;
        #[pallet::constant]
        type LiquidAlphaOn: Get<bool>;
        #[pallet::constant]
        type Yuma3On: Get<bool>;
        #[pallet::constant]
        type InitialColdkeySwapAnnouncementDelay: Get<BlockNumberFor<Self>>;
        #[pallet::constant]
        type InitialColdkeySwapReannouncementDelay: Get<BlockNumberFor<Self>>;
        #[pallet::constant]
        type InitialDissolveNetworkScheduleDuration: Get<BlockNumberFor<Self>>;
        #[pallet::constant]
        type InitialTaoWeight: Get<u64>;
        #[pallet::constant]
        type InitialEmaPriceHalvingPeriod: Get<u64>;
        #[pallet::constant]
        type InitialStartCallDelay: Get<u64>;
        #[pallet::constant]
        type KeySwapOnSubnetCost: Get<u64>;
        #[pallet::constant]
        type HotkeySwapOnSubnetInterval: Get<u64>;
        #[pallet::constant]
        type LeaseDividendsDistributionInterval: Get<BlockNumberFor<Self>>;
        #[pallet::constant]
        type MaxContributors: Get<u32>;
        #[pallet::constant]
        type MaxImmuneUidsPercentage: Get<sp_runtime::Percent>;
    }
}
