use frame_support::pallet_macros::pallet_section;

/// A [`pallet_section`] that defines the hooks for a pallet.
/// This can later be imported into the pallet using [`import_section`].
#[pallet_section]
mod hooks {
    use frame_system::pallet_prelude::BlockNumberFor;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_block_number: BlockNumberFor<T>) -> frame_support::weights::Weight {
            // Hooks stub — epoch/run_epoch stripped for Myosu genesis.
            frame_support::weights::Weight::from_parts(0, 0)
        }

        fn on_finalize(_block_number: BlockNumberFor<T>) {
            // No-op for Myosu genesis.
        }

        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            // No migrations needed for Myosu genesis.
            frame_support::weights::Weight::from_parts(0, 0)
        }
    }
}
