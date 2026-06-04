# ADR-014: Compile-time guard for `Stage0SwapInterface` `max_price` bound

- Status: Accepted
- Date: 2026-06-04
- Plan row: `NEM-002A` in `IMPLEMENTATION_PLAN.md` (and the lifted copy in
  the `nemesis/IMPLEMENTATION_PLAN.md` NEM-002A section).
- Specs: `specs/110426-security-posture.md` (slippage protection half).

## Context

`Stage0NoopSwap::max_price()` returns `u64::MAX` for any `C: Currency`
because the stage-0 no-op identity swap has no real market to constrain.
That value is consumed by the staking path
(`crates/myosu-chain/pallets/game-solver/src/staking/add_stake.rs:78`)
and the emission path
(`crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs:144`).
A live `TaoCurrency::MAX` price limit means a swap request can never be
denied for slippage, which is the documented stage-0 contract.

The risk is forward-looking: when stage-0 is replaced with a real AMM
impl, the implementer must remember to bound the price limit at a sane
ceiling. Forgetting to do so silently re-introduces the unbounded-
slippage vulnerability for every staking call and every emission call,
because both call sites already do `T::SwapInterface::stage0_max_price()`
without any upper-bound check. The current `DefaultPriceLimit` trait
gives the swap path a place to *request* a default limit, but it does
not give the runtime a way to *assert* that the bound is real.

## Decision

Add a **type-level** const-eval seam in
`crates/myosu-chain/pallets/swap-interface/src/lib.rs`:

- `pub const MAX_VALID_SWAP_PRICE_LIMIT: u64 = u64::MAX;` — the upper
  bound any `SwapHandler::max_price()` impl may return. Stage-0 is
  exempt and uses exactly this value.
- `pub const STRICT_MAX_VALID_SWAP_PRICE_LIMIT: u64 = u64::MAX / 2;` —
  the maximum a real (non-`Stage0NoopSwap`) AMM impl may return. Values
  above this indicate a missing bounded-price guard.
- `pub const fn check_strict_bound(bound: u64) -> bool` — the
  `<= STRICT_MAX_VALID_SWAP_PRICE_LIMIT` predicate in `const fn` form so
  future const-eval seams in the runtime can call it directly.
- `pub trait SwapPriceLimitBounded` with
  `const PRICE_LIMIT_BOUND: u64 = MAX_VALID_SWAP_PRICE_LIMIT` and a
  default-method `fn is_within_strict_bound() -> bool`. Real AMM impls
  override `PRICE_LIMIT_BOUND` to a value
  `<= STRICT_MAX_VALID_SWAP_PRICE_LIMIT`.

The `Stage0NoopSwap` impl in `crates/myosu-chain/runtime/src/lib.rs`
explicitly sets `PRICE_LIMIT_BOUND: u64 = MAX_VALID_SWAP_PRICE_LIMIT`,
and a const-eval `assert!` near the impl block requires the live bound
to be either the stage-0 opt-out or strictly within the strict ceiling.
A drop-the-impl-block refactor fails the `const _ : () = assert!(...)`
at compile time, not at runtime. The unit test
`stage0_noop_swap_price_limit_bound_is_max` makes the stage-0 opt-out
auditable in the test suite and cross-references this ADR.

The `DefaultPriceLimit` trait is left untouched: it remains the
slippage-protection seam the swap path already uses to *request* a
default limit, and `SwapPriceLimitBounded` is the additive type-level
guard on top of it. A future migration that adds a real AMM impl must
override `SwapPriceLimitBounded::PRICE_LIMIT_BOUND` to a value
`<= STRICT_MAX_VALID_SWAP_PRICE_LIMIT`; the const-eval check will then
fire at the next compile and the implementer will see the contract
broken in CI, not in production.

The e2e harness `tests/e2e/swap_price_limit_bound.sh` is the executable
end-to-end gate: it fail-closes if the public-API markers in the
swap-interface crate are missing, if the trait + default-method
shape drifts, if the `Stage0NoopSwap` impl block is dropped, if the
const-eval guard is removed, if the runtime unit test regresses, or
if this ADR is renamed or loses its cross-references.

## Alternatives considered

1. **Runtime `assert!` on every swap call.** A `frame_support::ensure!`
   on `T::SwapInterface::max_price::<C>() <= STRICT_...` would catch a
   regression at runtime, but the cost is paid on every swap call in
   the staking and emission hot paths. The seam is structural, not
   operational: a wrong bound is a *type* problem, not a *call* problem.

2. **Opaque internal flag.** A `BoundedPrice: bool` associated type on
   the `SwapHandler` trait would let impls opt out of the bound, but
   it hides the intent — a reviewer reading the impl block would have
   to remember which flag is the "stage-0 exemption" vs the "real
   AMM with a real bound". The const-eval seam with an explicit
   `PRICE_LIMIT_BOUND = MAX_VALID_SWAP_PRICE_LIMIT` override makes the
   intent auditable in the source.

3. **No seam at all.** Leave the runtime comment at
   `crates/myosu-chain/runtime/src/lib.rs:94-97` and rely on
   reviewer vigilance. Rejected because the comment already
   acknowledges the risk in prose, but nothing structurally prevents
   a future implementer from inheriting the same default and
   re-introducing the vulnerability. The seam is cheap (three
   constants, one trait, one impl block, one const-eval assert) and
   removes the prose-only contract.

## Consequences

- **Positive.** A future AMM migration that drops the
  `PRICE_LIMIT_BOUND` override fails the runtime's `const _ : () = ...`
  guard at compile time, so the missing-bound regression cannot reach
  CI green, let alone testnet.
- **Positive.** The stage-0 opt-out is explicit in the source
  (`PRICE_LIMIT_BOUND: u64 = MAX_VALID_SWAP_PRICE_LIMIT`) and
  documented in the test name
  (`stage0_noop_swap_price_limit_bound_is_max`), so the
  intent is auditable in the diff, not in a docstring.
- **Positive.** The e2e harness is fail-closed on marker drift, so the
  public API + the runtime impl + the test + the ADR cannot silently
  lose the seam.
- **Negative.** The trait adds a small additive surface to the
  `subtensor_swap_interface` crate, which downstream AMM impls will
  need to learn about. This is the cost of the structural guarantee.
- **Negative.** The strict ceiling `u64::MAX / 2` is a judgement call.
  A future migration that wants to express a more generous limit
  (e.g. `u64::MAX - 1` to use the full encoding range) will need to
  either pick a new strict ceiling (and update this ADR + the
  `STRICT_MAX_VALID_SWAP_PRICE_LIMIT` constant in lockstep) or accept
  that the seam is the bound on the upper end. The current
  `u64::MAX / 2` is conservative and matches the documented "real
  AMM has a market price, not a u64::MAX" rationale.

## References

- `crates/myosu-chain/pallets/swap-interface/src/lib.rs` — the trait
  + constants + unit tests.
- `crates/myosu-chain/runtime/src/lib.rs` — the `Stage0NoopSwap` impl
  + the const-eval guard + the `stage0_noop_swap_price_limit_bound_is_max`
  unit test.
- `tests/e2e/swap_price_limit_bound.sh` — the executable end-to-end gate.
- `IMPLEMENTATION_PLAN.md` `NEM-002A` — the plan row that this ADR
  records the decision for.
- `nemesis/IMPLEMENTATION_PLAN.md` NEM-002A — the original contract
  this row is lifted from.
- `specs/110426-security-posture.md` — the slippage-protection half
  of the security posture spec.
