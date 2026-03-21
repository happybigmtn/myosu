use frame_support::pallet_macros::pallet_section;

/// A [`pallet_section`] that defines genesis configuration.
/// This can later be imported into the pallet using [`import_section`].
#[pallet_section]
mod genesis {
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub dummy: T::AccountId,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                dummy: T::AccountId::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {}
}
