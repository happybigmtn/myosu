use crate::pallet::{Axons, Config, Error, Keys, NeuronCertificates, Pallet, Prometheus};
use crate::{AxonInfo, NetUid, NeuronCertificate, PrometheusInfo};
use frame_support::ensure;
use sp_runtime::DispatchError;

impl<T: Config> Pallet<T> {
    pub fn do_serve_axon(
        netuid: NetUid,
        hotkey: &T::AccountId,
        axon: AxonInfo,
        certificate: Option<NeuronCertificate>,
    ) -> Result<(), DispatchError> {
        ensure!(Self::subnet_exists(netuid), Error::<T>::SubnetNotFound);
        ensure!(
            Keys::<T>::contains_key(netuid, hotkey),
            Error::<T>::HotkeyNotRegisteredInSubnet
        );

        Self::validate_ip_inputs(axon.ip_type, axon.ip, axon.port)?;
        Axons::<T>::insert(netuid, hotkey, axon);

        if let Some(certificate) = certificate {
            NeuronCertificates::<T>::insert(netuid, hotkey, certificate);
        }

        Ok(())
    }

    pub fn do_serve_prometheus(
        netuid: NetUid,
        hotkey: &T::AccountId,
        info: PrometheusInfo,
    ) -> Result<(), DispatchError> {
        ensure!(Self::subnet_exists(netuid), Error::<T>::SubnetNotFound);
        ensure!(
            Keys::<T>::contains_key(netuid, hotkey),
            Error::<T>::HotkeyNotRegisteredInSubnet
        );

        Self::validate_ip_inputs(info.ip_type, info.ip, info.port)?;
        Prometheus::<T>::insert(netuid, hotkey, info);

        Ok(())
    }

    fn validate_ip_inputs(ip_type: u8, ip: u128, port: u16) -> Result<(), DispatchError> {
        ensure!(matches!(ip_type, 4 | 6), Error::<T>::InvalidIpType);
        ensure!(port != 0, Error::<T>::InvalidPort);

        let valid_ip = match ip_type {
            4 => ip != 0 && ip <= u32::MAX as u128,
            6 => ip != 0,
            _ => false,
        };

        ensure!(valid_ip, Error::<T>::InvalidIpAddress);
        Ok(())
    }
}
