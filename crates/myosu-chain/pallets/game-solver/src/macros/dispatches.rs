#![allow(clippy::crate_in_macro_def)]
use frame_support::pallet_macros::pallet_section;

/// A [`pallet_section`] that defines the dispatchable functions for a pallet.
/// This can later be imported into the pallet using [`import_section`].
#[pallet_section]
mod dispatches {
    /// Dispatchable functions allow users to interact with the pallet and invoke state changes.
    /// These functions materialize as "extrinsics", which are often compared to transactions.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // Dispatchable stubs — subtensor-specific calls stripped for Myosu genesis.
    }
}
