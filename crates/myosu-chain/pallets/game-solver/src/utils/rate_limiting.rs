use crate::{Config, Hyperparameter, NetUid, Pallet, RateLimitKey};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug, TypeInfo)]
pub enum TransactionType {
    SetChildren,
    SetChildkeyTake,
    Unknown,
    RegisterNetwork,
    SetWeightsVersionKey,
    SetSNOwnerHotkey,
    OwnerHyperparamUpdate(Hyperparameter),
    MechanismCountUpdate,
    MechanismEmission,
    MaxUidsTrimming,
    AddStakeBurn,
}

impl TransactionType {
    pub fn rate_limit(self) -> u64 {
        match self {
            Self::SetChildren => 150,
            Self::SetChildkeyTake => 150,
            Self::RegisterNetwork => 7_200,
            Self::MechanismCountUpdate => 100,
            Self::MechanismEmission => 100,
            Self::MaxUidsTrimming => 100,
            Self::Unknown
            | Self::SetWeightsVersionKey
            | Self::SetSNOwnerHotkey
            | Self::OwnerHyperparamUpdate(_)
            | Self::AddStakeBurn => 0,
        }
    }

    pub fn rate_limit_on_subnet(self, _netuid: NetUid) -> u64 {
        self.rate_limit()
    }

    pub fn passes_rate_limit<T: Config>(self, key: &T::AccountId) -> bool {
        self.passes_rate_limit_on_subnet::<T>(key, 0)
    }

    pub fn passes_rate_limit_on_subnet<T: Config>(
        self,
        key: &T::AccountId,
        netuid: NetUid,
    ) -> bool {
        let block = Pallet::<T>::get_current_block_as_u64();
        let last_block = self.last_block_on_subnet::<T>(key, netuid);
        last_block == 0 || block.saturating_sub(last_block) >= self.rate_limit_on_subnet(netuid)
    }

    pub fn last_block<T: Config>(self, key: &T::AccountId) -> u64 {
        self.last_block_on_subnet::<T>(key, 0)
    }

    pub fn last_block_on_subnet<T: Config>(self, key: &T::AccountId, netuid: NetUid) -> u64 {
        match self {
            Self::RegisterNetwork => Pallet::<T>::get_network_last_lock_block(),
            Self::SetSNOwnerHotkey => {
                Pallet::<T>::get_rate_limited_last_block(&RateLimitKey::SetSNOwnerHotkey(netuid))
            }
            Self::OwnerHyperparamUpdate(hparam) => Pallet::<T>::get_rate_limited_last_block(
                &RateLimitKey::OwnerHyperparamUpdate(netuid, hparam),
            ),
            _ => Pallet::<T>::get_rate_limited_last_block(&RateLimitKey::Transaction(
                key.clone(),
                netuid,
                self.into(),
            )),
        }
    }

    pub fn set_last_block_on_subnet<T: Config>(
        self,
        key: &T::AccountId,
        netuid: NetUid,
        block: u64,
    ) {
        match self {
            Self::RegisterNetwork => Pallet::<T>::set_network_last_lock_block(block),
            Self::SetSNOwnerHotkey => Pallet::<T>::set_rate_limited_last_block(
                &RateLimitKey::SetSNOwnerHotkey(netuid),
                block,
            ),
            Self::OwnerHyperparamUpdate(hparam) => Pallet::<T>::set_rate_limited_last_block(
                &RateLimitKey::OwnerHyperparamUpdate(netuid, hparam),
                block,
            ),
            _ => Pallet::<T>::set_rate_limited_last_block(
                &RateLimitKey::Transaction(key.clone(), netuid, self.into()),
                block,
            ),
        }
    }
}

impl From<TransactionType> for u16 {
    fn from(value: TransactionType) -> Self {
        match value {
            TransactionType::SetChildren => 0,
            TransactionType::SetChildkeyTake => 1,
            TransactionType::Unknown => 2,
            TransactionType::RegisterNetwork => 3,
            TransactionType::SetWeightsVersionKey => 4,
            TransactionType::SetSNOwnerHotkey => 5,
            TransactionType::OwnerHyperparamUpdate(_) => 6,
            TransactionType::MechanismCountUpdate => 7,
            TransactionType::MechanismEmission => 8,
            TransactionType::MaxUidsTrimming => 9,
            TransactionType::AddStakeBurn => 10,
        }
    }
}

impl From<u16> for TransactionType {
    fn from(value: u16) -> Self {
        match value {
            0 => TransactionType::SetChildren,
            1 => TransactionType::SetChildkeyTake,
            3 => TransactionType::RegisterNetwork,
            4 => TransactionType::SetWeightsVersionKey,
            5 => TransactionType::SetSNOwnerHotkey,
            6 => TransactionType::OwnerHyperparamUpdate(Hyperparameter::Unknown),
            7 => TransactionType::MechanismCountUpdate,
            8 => TransactionType::MechanismEmission,
            9 => TransactionType::MaxUidsTrimming,
            10 => TransactionType::AddStakeBurn,
            _ => TransactionType::Unknown,
        }
    }
}

impl From<Hyperparameter> for TransactionType {
    fn from(value: Hyperparameter) -> Self {
        Self::OwnerHyperparamUpdate(value)
    }
}

impl<T: Config> Pallet<T> {
    pub fn remove_last_tx_block(key: &T::AccountId) {
        Self::remove_rate_limited_last_block(&RateLimitKey::LastTxBlock(key.clone()))
    }

    pub fn set_last_tx_block(key: &T::AccountId, block: u64) {
        Self::set_rate_limited_last_block(&RateLimitKey::LastTxBlock(key.clone()), block);
    }

    pub fn get_last_tx_block(key: &T::AccountId) -> u64 {
        Self::get_rate_limited_last_block(&RateLimitKey::LastTxBlock(key.clone()))
    }

    pub fn remove_last_tx_block_delegate_take(key: &T::AccountId) {
        Self::remove_rate_limited_last_block(&RateLimitKey::LastTxBlockDelegateTake(key.clone()))
    }

    pub fn set_last_tx_block_delegate_take(key: &T::AccountId, block: u64) {
        Self::set_rate_limited_last_block(
            &RateLimitKey::LastTxBlockDelegateTake(key.clone()),
            block,
        );
    }

    pub fn get_last_tx_block_delegate_take(key: &T::AccountId) -> u64 {
        Self::get_rate_limited_last_block(&RateLimitKey::LastTxBlockDelegateTake(key.clone()))
    }

    pub fn remove_last_tx_block_childkey(key: &T::AccountId) {
        Self::remove_rate_limited_last_block(&RateLimitKey::LastTxBlockChildKeyTake(key.clone()))
    }

    pub fn set_last_tx_block_childkey(key: &T::AccountId, block: u64) {
        Self::set_rate_limited_last_block(
            &RateLimitKey::LastTxBlockChildKeyTake(key.clone()),
            block,
        );
    }

    pub fn get_last_tx_block_childkey_take(key: &T::AccountId) -> u64 {
        Self::get_rate_limited_last_block(&RateLimitKey::LastTxBlockChildKeyTake(key.clone()))
    }

    pub fn exceeds_tx_rate_limit(prev_tx_block: u64, current_block: u64) -> bool {
        if prev_tx_block == 0 {
            return false;
        }

        current_block.saturating_sub(prev_tx_block) <= TransactionType::SetChildren.rate_limit()
    }

    pub fn exceeds_tx_delegate_take_rate_limit(prev_tx_block: u64, current_block: u64) -> bool {
        if prev_tx_block == 0 {
            return false;
        }

        current_block.saturating_sub(prev_tx_block)
            <= TransactionType::SetChildkeyTake.rate_limit()
    }
}
