use crate::NetUid;
use crate::pallet::{
    Config, Delegates, Error, Keys, NextNeuronUid, NextSubnetUid, Owner, Pallet, SubnetOwner,
};
use frame_support::ensure;
use sp_runtime::DispatchError;

impl<T: Config> Pallet<T> {
    pub fn do_register_subnet(owner: &T::AccountId) -> Result<NetUid, DispatchError> {
        let netuid = NextSubnetUid::<T>::get();
        let next_netuid = netuid.checked_add(1).ok_or(Error::<T>::NetUidExhausted)?;

        SubnetOwner::<T>::insert(netuid, owner.clone());
        NextSubnetUid::<T>::put(next_netuid);

        Ok(netuid)
    }

    pub fn do_register_hotkey(
        netuid: NetUid,
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
    ) -> Result<u16, DispatchError> {
        ensure!(Self::subnet_exists(netuid), Error::<T>::SubnetNotFound);
        ensure!(
            !Keys::<T>::contains_key(netuid, hotkey),
            Error::<T>::HotkeyAlreadyRegistered
        );

        if let Some(existing_owner) = Owner::<T>::get(hotkey) {
            ensure!(
                existing_owner == *coldkey,
                Error::<T>::HotkeyOwnedByDifferentColdkey
            );
        } else {
            Owner::<T>::insert(hotkey, coldkey.clone());
        }

        if !Delegates::<T>::contains_key(hotkey) {
            Delegates::<T>::insert(hotkey, 0);
        }

        let next_uid = NextNeuronUid::<T>::get(netuid);
        let following_uid = next_uid.checked_add(1).ok_or(Error::<T>::NetUidExhausted)?;

        Keys::<T>::insert(netuid, hotkey, next_uid);
        NextNeuronUid::<T>::insert(netuid, following_uid);

        Ok(next_uid)
    }
}
