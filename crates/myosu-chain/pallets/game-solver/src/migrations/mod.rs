//! Stage 0 migration stubs.
//!
//! The carried migration tree is historical subtensor upgrade baggage. It is a
//! large compile fanout and not part of the first honest restart slice for the
//! Myosu pallet. The restart path keeps this module as a stable namespace while
//! runtime upgrades are reduced to a no-op.

use super::*;
use frame_support::pallet_prelude::Weight;

// Keep the runtime's live total-issuance repair under the GameSolver namespace
// while the rest of the historical migration tree stays stripped in stage 0.
pub mod migrate_create_root_network;
pub mod migrate_init_total_issuance;

/// Stage 0 helper retained so call sites can keep a stable migration namespace
/// while the historical upgrade tree remains stripped.
#[allow(dead_code)]
pub(crate) fn migrate_storage<T: frame_system::Config>(
    _migration_name: &'static str,
    _pallet_name: &'static str,
    _storage_name: &'static str,
) -> Weight {
    let _ = sp_std::marker::PhantomData::<T>;
    Weight::zero()
}

/// Stage 0 prefix-removal helper retained as a no-op.
#[allow(dead_code)]
pub(crate) fn remove_prefix<T: frame_system::Config>(
    _module: &str,
    _old_map: &str,
    weight: &mut Weight,
) {
    let _ = sp_std::marker::PhantomData::<T>;
    *weight = weight.saturating_add(Weight::zero());
}
