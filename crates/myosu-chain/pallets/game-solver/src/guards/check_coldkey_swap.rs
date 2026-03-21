//! Phase 1 restart placeholder for the coldkey swap dispatch guard.
//!
//! The subtensor implementation depended on `DispatchGuard` and several pallet
//! surfaces that are intentionally stripped from the Myosu restart slice.
//! This no-op guard keeps the module shape intact until the pallet regains
//! the specific call paths that need policy enforcement.

use sp_std::marker::PhantomData;

pub struct CheckColdkeySwap<T>(PhantomData<T>);

impl<T> CheckColdkeySwap<T> {
    pub fn allows_call<Origin, Call>(_origin: &Origin, _call: &Call) -> bool {
        true
    }
}
