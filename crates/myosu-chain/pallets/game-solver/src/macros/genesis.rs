use frame_support::pallet_macros::pallet_section;

/// A [`pallet_section`] that defines the genesis configuration for a pallet.
/// This can later be imported into the pallet using [`import_section`].
#[pallet_section]
mod genesis {
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
        fn build(&self) {
            // Genesis build stub — full subnet initialization deferred to runtime genesis.
        }
    }
}
