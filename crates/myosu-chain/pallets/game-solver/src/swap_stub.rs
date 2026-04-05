//! No-op swap stub for myosu's single-token model.
//!
//! Subtensor's swap pallet implements a full AMM with Alpha/TAO token pairs.
//! Myosu uses a single token (MYOSU) with no AMM. This stub satisfies
//! the 37 callsites across registration, staking, and emission that require
//! SwapHandler + SwapEngine trait bounds on pallet Config.
//!
//! All swaps are identity: amount_in == amount_out, zero fees.

use core::fmt::Debug;
use core::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwapResult<Balance> {
    pub amount_in: Balance,
    pub amount_out: Balance,
    pub fee: Balance,
}

/// Direction marker: TAO to Alpha.
pub struct GetAlphaForTao<T>(PhantomData<T>);

/// Direction marker: Alpha to TAO.
pub struct GetTaoForAlpha<T>(PhantomData<T>);

/// Numeric type usable as a swap balance.
pub trait SwapBalance: Default + Copy + Ord + Debug {
    fn zero() -> Self;
    fn max_value() -> Self;
    fn one() -> Self;
}

impl SwapBalance for u64 {
    fn zero() -> Self { 0 }
    fn max_value() -> Self { u64::MAX }
    fn one() -> Self { 1 }
}

impl SwapBalance for u128 {
    fn zero() -> Self { 0 }
    fn max_value() -> Self { u128::MAX }
    fn one() -> Self { 1 }
}

/// Default price limit for swap operations.
pub trait DefaultPriceLimit {
    type Balance: SwapBalance;
    fn default_price_limit() -> Self::Balance;
}

/// Core swap handler trait covering all AMM-adjacent operations.
///
/// In subtensor this backs a full constant-product AMM. In myosu the
/// single-token model means every method is either identity or no-op.
pub trait SwapHandler {
    type Balance: SwapBalance;

    fn swap(netuid: u16, amount: Self::Balance) -> Self::Balance;
    fn sim_swap(netuid: u16, amount: Self::Balance) -> Self::Balance;
    fn approx_fee_amount(netuid: u16, amount: Self::Balance) -> Self::Balance;
    fn current_alpha_price(netuid: u16) -> Self::Balance;
    fn get_protocol_tao(netuid: u16) -> Self::Balance;
    /// Returns the maximum acceptable execution price for swap callers.
    ///
    /// The stage-0 identity stub intentionally returns `Balance::max_value()`,
    /// which disables slippage protection. That is acceptable only while swaps
    /// are a no-op compatibility seam and must be revisited before any
    /// mainnet-style token economics ship.
    fn max_price(netuid: u16) -> Self::Balance;
    fn min_price(netuid: u16) -> Self::Balance;
    fn adjust_protocol_liquidity(netuid: u16) -> Result<(), ()>;
    fn is_user_liquidity_enabled(netuid: u16) -> bool;
    fn dissolve_all_liquidity_providers(netuid: u16) -> Result<(), ()>;
    fn clear_protocol_liquidity(netuid: u16) -> Result<(), ()>;
    fn toggle_user_liquidity(netuid: u16);
}

/// Swap engine for a specific directional conversion.
pub trait SwapEngine<Direction> {
    type Balance: SwapBalance;

    fn swap_engine(netuid: u16, amount: Self::Balance) -> Result<SwapResult<Self::Balance>, ()>;
}

/// No-op swap: identity conversion, zero fees, no pool state.
pub struct NoOpSwap<B>(PhantomData<B>);

impl<B: SwapBalance> DefaultPriceLimit for NoOpSwap<B> {
    type Balance = B;
    fn default_price_limit() -> B { B::max_value() }
}

impl<B: SwapBalance> SwapHandler for NoOpSwap<B> {
    type Balance = B;

    fn swap(_netuid: u16, amount: B) -> B { amount }
    fn sim_swap(_netuid: u16, amount: B) -> B { amount }
    fn approx_fee_amount(_netuid: u16, _amount: B) -> B { B::zero() }
    fn current_alpha_price(_netuid: u16) -> B { B::one() }
    fn get_protocol_tao(_netuid: u16) -> B { B::zero() }
    fn max_price(_netuid: u16) -> B {
        // Stage-0 keeps an effectively infinite ceiling because the no-op swap
        // is a 1:1 identity conversion. This must not survive a real AMM path.
        B::max_value()
    }
    fn min_price(_netuid: u16) -> B { B::zero() }
    fn adjust_protocol_liquidity(_netuid: u16) -> Result<(), ()> { Ok(()) }
    fn is_user_liquidity_enabled(_netuid: u16) -> bool { false }
    fn dissolve_all_liquidity_providers(_netuid: u16) -> Result<(), ()> { Ok(()) }
    fn clear_protocol_liquidity(_netuid: u16) -> Result<(), ()> { Ok(()) }
    fn toggle_user_liquidity(_netuid: u16) {}
}

impl<B: SwapBalance, D> SwapEngine<D> for NoOpSwap<B> {
    type Balance = B;

    fn swap_engine(_netuid: u16, amount: B) -> Result<SwapResult<B>, ()> {
        Ok(SwapResult { amount_in: amount, amount_out: amount, fee: B::zero() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Stub = NoOpSwap<u64>;

    #[test]
    fn identity_swap() {
        assert_eq!(Stub::swap(1, 100), 100);
        assert_eq!(Stub::sim_swap(1, 100), 100);
        assert_eq!(Stub::swap(1, 0), 0);
        assert_eq!(Stub::swap(1, u64::MAX), u64::MAX);
    }

    #[test]
    fn registration_uses_stub() {
        // Registration path: check max_price then swap. Identity stub has
        // no price ceiling and returns amount unchanged (direct burn).
        let max = Stub::max_price(1);
        assert_eq!(max, u64::MAX);
        let burned = Stub::swap(1, 1_000_000);
        assert_eq!(burned, 1_000_000);
    }

    #[test]
    fn zero_fees() {
        assert_eq!(Stub::approx_fee_amount(1, 1000), 0);
    }

    #[test]
    fn identity_price() {
        assert_eq!(Stub::current_alpha_price(1), 1);
    }

    #[test]
    fn no_protocol_tao() {
        assert_eq!(Stub::get_protocol_tao(1), 0);
    }

    #[test]
    fn liquidity_disabled() {
        assert!(!Stub::is_user_liquidity_enabled(1));
    }

    #[test]
    fn liquidity_ops_succeed() {
        assert!(Stub::adjust_protocol_liquidity(1).is_ok());
        assert!(Stub::dissolve_all_liquidity_providers(1).is_ok());
        assert!(Stub::clear_protocol_liquidity(1).is_ok());
    }

    #[test]
    fn swap_engine_identity() {
        let result = <Stub as SwapEngine<GetAlphaForTao<()>>>::swap_engine(1, 100).unwrap();
        assert_eq!(result.amount_in, 100);
        assert_eq!(result.amount_out, 100);
        assert_eq!(result.fee, 0);

        let result = <Stub as SwapEngine<GetTaoForAlpha<()>>>::swap_engine(1, 50).unwrap();
        assert_eq!(result.amount_out, 50);
    }

    #[test]
    fn default_price_limit_is_max() {
        assert_eq!(Stub::default_price_limit(), u64::MAX);
    }
}
