use frame_support::pallet_macros::pallet_section;

/// A [`pallet_section`] that defines hooks for the pallet.
/// This can later be imported into the pallet using [`import_section`].
#[pallet_section]
mod hooks {
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
}
