use frame_support::pallet_macros::pallet_section;

/// A [`pallet_section`] that defines the events for a pallet.
/// This can later be imported into the pallet using [`import_section`].
#[pallet_section]
mod hooks {
    // ================
    // ==== Hooks =====
    // ================
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // ---- Called on the initialization of this pallet. (the order of on_finalize calls is determined in the runtime)
        //
        // # Args:
        // 	* 'n': (BlockNumberFor<T>):
        // 		- The number of the block we are initializing.
        fn on_initialize(block_number: BlockNumberFor<T>) -> Weight {
            let hotkey_swap_clean_up_weight = Self::clean_up_hotkey_swap_records(block_number);

            let block_step_result = Self::block_step();
            match block_step_result {
                Ok(_) => {
                    // --- If the block step was successful, return the weight.
                    log::debug!("Successfully ran block step.");
                    Weight::from_parts(110_634_229_000_u64, 0)
                        .saturating_add(T::DbWeight::get().reads(8304_u64))
                        .saturating_add(T::DbWeight::get().writes(110_u64))
                        .saturating_add(hotkey_swap_clean_up_weight)
                }
                Err(e) => {
                    // --- If the block step was unsuccessful, return the weight anyway.
                    log::error!("Error while stepping block: {:?}", e);
                    Weight::from_parts(110_634_229_000_u64, 0)
                        .saturating_add(T::DbWeight::get().reads(8304_u64))
                        .saturating_add(T::DbWeight::get().writes(110_u64))
                        .saturating_add(hotkey_swap_clean_up_weight)
                }
            }
        }

        // ---- Called on the finalization of this pallet. The code weight must be taken into account prior to the execution of this macro.
        //
        // # Args:
        // 	* 'n': (BlockNumberFor<T>):
        // 		- The number of the block we are finalizing.
        fn on_finalize(_block_number: BlockNumberFor<T>) {
            for _ in StakingOperationRateLimiter::<T>::drain() {
                // Clear all entries each block
            }
        }

        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            frame_support::weights::Weight::zero()
        }

        #[cfg(feature = "try-runtime")]
        fn try_state(_n: BlockNumberFor<T>) -> Result<(), sp_runtime::TryRuntimeError> {
            Self::check_total_issuance()?;
            // Disabled: https://github.com/opentensor/subtensor/pull/1166
            // Self::check_total_stake()?;
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        // This function is to clean up the old hotkey swap records
        // It just clean up for one subnet at a time, according to the block number
        fn clean_up_hotkey_swap_records(block_number: BlockNumberFor<T>) -> Weight {
            let mut weight = Weight::from_parts(0, 0);
            let hotkey_swap_on_subnet_interval = T::HotkeySwapOnSubnetInterval::get();
            let block_number: u64 = TryInto::try_into(block_number)
                .ok()
                .expect("blockchain will not exceed 2^64 blocks; QED.");
            weight.saturating_accrue(T::DbWeight::get().reads(2_u64));

            let netuids = Self::get_all_subnet_netuids();
            weight.saturating_accrue(T::DbWeight::get().reads(netuids.len() as u64));

            if let Some(slot) = block_number.checked_rem(hotkey_swap_on_subnet_interval) {
                // only handle the subnet with the same residue as current block number by HotkeySwapOnSubnetInterval
                for netuid in netuids.iter().filter(|netuid| {
                    (u16::from(**netuid) as u64).checked_rem(hotkey_swap_on_subnet_interval)
                        == Some(slot)
                }) {
                    // Iterate over all the coldkeys in the subnet
                    for (coldkey, swap_block_number) in
                        LastHotkeySwapOnNetuid::<T>::iter_prefix(netuid)
                    {
                        // Clean up out of date swap records
                        if swap_block_number.saturating_add(hotkey_swap_on_subnet_interval)
                            < block_number
                        {
                            LastHotkeySwapOnNetuid::<T>::remove(netuid, coldkey);
                            weight.saturating_accrue(T::DbWeight::get().writes(1_u64));
                        }
                        weight.saturating_accrue(T::DbWeight::get().reads(1_u64));
                    }
                }
            }
            weight
        }
    }
}
