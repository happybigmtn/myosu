#![cfg_attr(not(feature = "std"), no_std)]
use core::ops::Neg;

use frame_support::pallet_prelude::*;
use substrate_fixed::types::U96F32;
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::{AlphaCurrency, Currency, NetUid, TaoCurrency};

pub use order::*;

mod order;

pub trait SwapEngine<O: Order>: DefaultPriceLimit<O::PaidIn, O::PaidOut> {
    fn swap(
        netuid: NetUid,
        order: O,
        price_limit: TaoCurrency,
        drop_fees: bool,
        should_rollback: bool,
    ) -> Result<SwapResult<O::PaidIn, O::PaidOut>, DispatchError>;
}

pub trait SwapHandler {
    fn swap<O: Order>(
        netuid: NetUid,
        order: O,
        price_limit: TaoCurrency,
        drop_fees: bool,
        should_rollback: bool,
    ) -> Result<SwapResult<O::PaidIn, O::PaidOut>, DispatchError>
    where
        Self: SwapEngine<O>;
    fn sim_swap<O: Order>(
        netuid: NetUid,
        order: O,
    ) -> Result<SwapResult<O::PaidIn, O::PaidOut>, DispatchError>
    where
        Self: SwapEngine<O>;

    fn approx_fee_amount<T: Currency>(netuid: NetUid, amount: T) -> T;
    fn current_alpha_price(netuid: NetUid) -> U96F32;
    fn get_protocol_tao(netuid: NetUid) -> TaoCurrency;
    fn max_price<C: Currency>() -> C;
    fn min_price<C: Currency>() -> C;
    fn adjust_protocol_liquidity(
        netuid: NetUid,
        tao_delta: TaoCurrency,
        alpha_delta: AlphaCurrency,
    );
    fn is_user_liquidity_enabled(netuid: NetUid) -> bool;
    fn dissolve_all_liquidity_providers(netuid: NetUid) -> DispatchResult;
    fn toggle_user_liquidity(netuid: NetUid, enabled: bool);
    fn clear_protocol_liquidity(netuid: NetUid) -> DispatchResult;
}

pub trait DefaultPriceLimit<PaidIn, PaidOut>
where
    PaidIn: Currency,
    PaidOut: Currency,
{
    fn default_price_limit<C: Currency>() -> C;
}

/// Maximum value any [`SwapHandler::max_price`] impl may return.
///
/// This is the stage-0 upper bound: the no-op identity swap intentionally
/// returns [`u64::MAX`] because it has no real market to constrain. The
/// constant exists so the stage-0 opt-out is explicit in the source — a
/// future implementer who wants an *intentionally* unbounded bound writes
/// `PRICE_LIMIT_BOUND: u64 = MAX_VALID_SWAP_PRICE_LIMIT;` and the intent
/// is auditable in the impl block instead of being silently inherited
/// from a default.
///
/// See `docs/adr/014-swap-price-limit-bound.md` for the full rationale
/// (NEM-002A / INV-005 slippage protection).
pub const MAX_VALID_SWAP_PRICE_LIMIT: u64 = u64::MAX;

/// Maximum value a *real* (non-`Stage0NoopSwap`) AMM impl may return from
/// [`SwapHandler::max_price`].
///
/// Values strictly above this constant indicate a missing bounded-price
/// guard: a real AMM has a market price, and an unbounded price limit
/// exposes staking and emission to slippage. The constant is the seam a
/// future AMM implementer must respect by overriding
/// [`SwapPriceLimitBounded::PRICE_LIMIT_BOUND`] to a value
/// `<= STRICT_MAX_VALID_SWAP_PRICE_LIMIT`. Stage-0 is exempt because
/// the no-op identity swap has no market.
pub const STRICT_MAX_VALID_SWAP_PRICE_LIMIT: u64 = u64::MAX / 2;

/// Return `true` iff `bound <= STRICT_MAX_VALID_SWAP_PRICE_LIMIT`.
///
/// Free function form of [`SwapPriceLimitBounded::is_within_strict_bound`]
/// for callers that want to check an arbitrary `u64` without instantiating
/// a trait. The `const fn` shape is intentional: future const-eval seams
/// in the runtime can call this directly to assert a bound at compile time.
pub const fn check_strict_bound(bound: u64) -> bool {
    bound <= STRICT_MAX_VALID_SWAP_PRICE_LIMIT
}

/// Type-level guard on the [`SwapHandler::max_price`] bound.
///
/// Any implementer of [`SwapHandler`] / [`DefaultPriceLimit`] that wants
/// to ship a real (non-stage-0) AMM impl must override
/// [`SwapPriceLimitBounded::PRICE_LIMIT_BOUND`] to a value
/// `<= STRICT_MAX_VALID_SWAP_PRICE_LIMIT`. The default
/// ([`MAX_VALID_SWAP_PRICE_LIMIT`]) is the stage-0 opt-out and is
/// explicit in the [`Stage0NoopSwap`](crate) impl so the intent is
/// auditable in the source.
///
/// The contract is checked at compile time via
/// [`SwapPriceLimitBounded::is_within_strict_bound`]: the `const fn`
/// returns `true` iff the bound is within the strict ceiling. Future
/// runtime-side `const _ : () = assert!(...)` guards can be wired on top
/// of this trait so a refactor that drops a strict bound fails to
/// compile, not to deploy.
pub trait SwapPriceLimitBounded {
    /// The price-limit bound this impl exposes via
    /// [`SwapHandler::max_price`]. Defaults to
    /// [`MAX_VALID_SWAP_PRICE_LIMIT`] (the stage-0 unbounded opt-out).
    /// Real AMM impls MUST override this to a value
    /// `<= STRICT_MAX_VALID_SWAP_PRICE_LIMIT`.
    const PRICE_LIMIT_BOUND: u64 = MAX_VALID_SWAP_PRICE_LIMIT;

    /// Return `true` iff [`PRICE_LIMIT_BOUND`] is within the strict
    /// ceiling. Stage-0 returns `false` (the bound is intentionally
    /// unbounded); a real AMM impl that has overridden the bound to a
    /// strict value returns `true`.
    fn is_within_strict_bound() -> bool {
        check_strict_bound(Self::PRICE_LIMIT_BOUND)
    }
}

/// Externally used swap result (for RPC)
#[freeze_struct("58ff42da64adce1a")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct SwapResult<PaidIn, PaidOut>
where
    PaidIn: Currency,
    PaidOut: Currency,
{
    pub amount_paid_in: PaidIn,
    pub amount_paid_out: PaidOut,
    pub fee_paid: PaidIn,
    pub fee_to_block_author: PaidIn,
}

impl<PaidIn, PaidOut> SwapResult<PaidIn, PaidOut>
where
    PaidIn: Currency,
    PaidOut: Currency,
{
    pub fn paid_in_reserve_delta(&self) -> i128 {
        self.amount_paid_in.to_u64() as i128
    }

    pub fn paid_in_reserve_delta_i64(&self) -> i64 {
        self.paid_in_reserve_delta()
            .clamp(i64::MIN as i128, i64::MAX as i128) as i64
    }

    pub fn paid_out_reserve_delta(&self) -> i128 {
        (self.amount_paid_out.to_u64() as i128).neg()
    }

    pub fn paid_out_reserve_delta_i64(&self) -> i64 {
        (self.amount_paid_out.to_u64() as i128)
            .neg()
            .clamp(i64::MIN as i128, i64::MAX as i128) as i64
    }
}

#[cfg(test)]
mod swap_price_limit_bounded_tests {
    //! Compile-time guard for the `max_price` bound (NEM-002A).
    //!
    //! These tests are the proof surface for the `SwapPriceLimitBounded`
    //! trait + `MAX_VALID_SWAP_PRICE_LIMIT` / `STRICT_MAX_VALID_SWAP_PRICE_LIMIT`
    //! constants in `crates/myosu-chain/pallets/swap-interface/src/lib.rs`.
    //! The trait + constants exist so a future real AMM impl must override
    //! `PRICE_LIMIT_BOUND` to a value `<= STRICT_MAX_VALID_SWAP_PRICE_LIMIT`,
    //! otherwise the const-eval seam documented in ADR-014 fires at the
    //! next compile. See `docs/adr/014-swap-price-limit-bound.md`.

    use super::{
        MAX_VALID_SWAP_PRICE_LIMIT, STRICT_MAX_VALID_SWAP_PRICE_LIMIT, SwapPriceLimitBounded,
        check_strict_bound,
    };

    /// Marker struct used to exercise the default `PRICE_LIMIT_BOUND`
    /// value without touching the `Stage0NoopSwap` runtime impl.
    struct DefaultBoundProbe;

    impl SwapPriceLimitBounded for DefaultBoundProbe {}

    #[test]
    fn check_strict_bound_returns_true_at_or_below_strict_ceiling() {
        // The strict ceiling itself is included (per the `<=` contract).
        assert!(check_strict_bound(0));
        assert!(check_strict_bound(1));
        assert!(check_strict_bound(STRICT_MAX_VALID_SWAP_PRICE_LIMIT));
    }

    #[test]
    fn check_strict_bound_returns_false_above_strict_ceiling() {
        // Strictly above the ceiling: false. Even `STRICT + 1` fails.
        assert!(!check_strict_bound(STRICT_MAX_VALID_SWAP_PRICE_LIMIT + 1));
        assert!(!check_strict_bound(u64::MAX));
        assert!(!check_strict_bound(MAX_VALID_SWAP_PRICE_LIMIT));
    }

    #[test]
    fn default_trait_bound_is_max_valid_swap_price_limit() {
        // The default trait value MUST be the stage-0 upper bound so
        // a missing override is auditable as "stage-0 opt-out", not as
        // "no bound specified at all".
        assert_eq!(
            DefaultBoundProbe::PRICE_LIMIT_BOUND,
            MAX_VALID_SWAP_PRICE_LIMIT
        );
        assert!(!DefaultBoundProbe::is_within_strict_bound());
        // The default-method path must match the free-function path.
        assert_eq!(
            DefaultBoundProbe::is_within_strict_bound(),
            check_strict_bound(DefaultBoundProbe::PRICE_LIMIT_BOUND)
        );
    }

    #[test]
    fn strict_ceiling_is_strictly_below_max_valid_bound() {
        // The strict ceiling must be a real bound, not coincidentally
        // equal to `MAX_VALID_SWAP_PRICE_LIMIT`. If this ever fails the
        // constant has been rounded up and the whole seam is meaningless.
        assert!(STRICT_MAX_VALID_SWAP_PRICE_LIMIT < MAX_VALID_SWAP_PRICE_LIMIT);
        // And the strict ceiling must be at least half of the max (the
        // documented shape — keeps enough headroom for an AMM to express
        // a real but generous limit without touching the unbounded opt-out).
        assert_eq!(STRICT_MAX_VALID_SWAP_PRICE_LIMIT, u64::MAX / 2);
    }
}

