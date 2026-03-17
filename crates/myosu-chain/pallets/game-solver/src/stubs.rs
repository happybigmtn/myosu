//! No-op stubs for subtensor trait dependencies.
//!
//! Subtensor's runtime depends on several traits from other pallets that
//! myosu does not need. These no-op stubs satisfy the trait bounds while
//! keeping the runtime minimal.

use alloc::vec::Vec;
use core::marker::PhantomData;

/// No-op proxy interface stub.
///
/// Subtensor uses pallet_proxy for account delegation and multi-sig.
/// Myosu has no proxy functionality in stage 0.
pub struct ProxyStub<T>(pub PhantomData<T>);

impl<T> ProxyStub<T> {
    /// Always returns false - no proxies exist.
    pub fn exists(_delegate: &T) -> bool {
        false
    }

    /// Returns the original account - no delegation.
    pub fn proxied(_who: &T) -> Option<T> {
        None
    }

    /// No-op proxy check - always returns the original account.
    pub fn real(_who: T) -> T {
        _who
    }

    /// Always returns false - no pure proxies.
    pub fn is_pure(_who: &T) -> bool {
        false
    }
}

/// Trait for proxy operations.
///
/// Mirrors subtensor's ProxyInterface trait without the dependency.
pub trait ProxyInterface<AccountId> {
    fn exists(delegate: &AccountId) -> bool;
    fn proxied(who: &AccountId) -> Option<AccountId>;
    fn real(who: AccountId) -> AccountId;
    fn is_pure(who: &AccountId) -> bool;
}

impl<T, AccountId> ProxyInterface<AccountId> for ProxyStub<T> {
    fn exists(_delegate: &AccountId) -> bool {
        false
    }

    fn proxied(_who: &AccountId) -> Option<AccountId> {
        None
    }

    fn real(who: AccountId) -> AccountId {
        who
    }

    fn is_pure(_who: &AccountId) -> bool {
        false
    }
}

/// No-op commitments interface stub.
///
/// Subtensor uses pallet_commitments for arbitrary data commitments.
/// Myosu does not need general-purpose commitments in stage 0.
pub struct CommitmentsStub<T>(pub PhantomData<T>);

impl<T> CommitmentsStub<T> {
    /// No-op set commitment - always succeeds.
    pub fn set_commitment(_who: &T, _data: &[u8]) -> Result<(), ()> {
        Ok(())
    }

    /// Always returns None - no commitments stored.
    pub fn get_commitment(_who: &T) -> Option<Vec<u8>> {
        None
    }

    /// Always returns 0 - no rate limiting.
    pub fn rate_limit() -> u64 {
        0
    }
}

/// Trait for commitment operations.
///
/// Mirrors subtensor's CommitmentsInterface trait.
pub trait CommitmentsInterface<AccountId> {
    fn set_commitment(who: &AccountId, data: &[u8]) -> Result<(), ()>;
    fn get_commitment(who: &AccountId) -> Option<Vec<u8>>;
    fn rate_limit() -> u64;
}

impl<T, AccountId> CommitmentsInterface<AccountId> for CommitmentsStub<T> {
    fn set_commitment(_who: &AccountId, _data: &[u8]) -> Result<(), ()> {
        Ok(())
    }

    fn get_commitment(_who: &AccountId) -> Option<Vec<u8>> {
        None
    }

    fn rate_limit() -> u64 {
        0
    }
}

/// No-op authorship provider stub.
///
/// Subtensor uses pallet_authorship for block author tracking.
/// Myosu does not need special authorship handling in stage 0.
pub struct AuthorshipStub<T>(pub PhantomData<T>);

impl<T> AuthorshipStub<T> {
    /// Always returns None - no author tracked.
    pub fn author() -> Option<T> {
        None
    }

    /// Always returns None - no uncles.
    pub fn uncles() -> Vec<T> {
        Vec::new()
    }

    /// No-op - no authorship set.
    pub fn set_author(_author: &T) {}
}

/// Trait for authorship operations.
///
/// Mirrors subtensor's AuthorshipProvider trait.
pub trait AuthorshipProvider<AccountId> {
    fn author() -> Option<AccountId>;
    fn uncles() -> Vec<AccountId>;
    fn set_author(author: &AccountId);
}

impl<T, AccountId> AuthorshipProvider<AccountId> for AuthorshipStub<T> {
    fn author() -> Option<AccountId> {
        None
    }

    fn uncles() -> Vec<AccountId> {
        Vec::new()
    }

    fn set_author(_author: &AccountId) {}
}

/// No-op coldkey swap check stub.
///
/// Subtensor has coldkey swap functionality for account recovery.
/// Myosu does not support coldkey swaps in stage 0.
pub struct ColdkeySwapStub<T>(pub PhantomData<T>);

impl<T> ColdkeySwapStub<T> {
    /// Always returns false - no swaps allowed.
    pub fn is_coldkey_swap(_coldkey: &T) -> bool {
        false
    }

    /// Always returns false - no hotkey scheduled.
    pub fn is_hotkey_schedule_swap(_hotkey: &T) -> bool {
        false
    }

    /// Always returns 0 - no schedule exists.
    pub fn coldkey_swap_block(_coldkey: &T) -> Option<u64> {
        None
    }

    /// No-op - cannot perform swaps.
    pub fn do_coldkey_swap(_old_coldkey: &T, _new_coldkey: &T) -> Result<(), ()> {
        Err(())
    }
}

/// Trait for coldkey swap operations.
///
/// Mirrors subtensor's CheckColdkeySwap trait.
pub trait CheckColdkeySwap<AccountId> {
    fn is_coldkey_swap(coldkey: &AccountId) -> bool;
    fn is_hotkey_schedule_swap(hotkey: &AccountId) -> bool;
    fn coldkey_swap_block(coldkey: &AccountId) -> Option<u64>;
    fn do_coldkey_swap(old_coldkey: &AccountId, new_coldkey: &AccountId) -> Result<(), ()>;
}

impl<T, AccountId> CheckColdkeySwap<AccountId> for ColdkeySwapStub<T> {
    fn is_coldkey_swap(_coldkey: &AccountId) -> bool {
        false
    }

    fn is_hotkey_schedule_swap(_hotkey: &AccountId) -> bool {
        false
    }

    fn coldkey_swap_block(_coldkey: &AccountId) -> Option<u64> {
        None
    }

    fn do_coldkey_swap(_old_coldkey: &AccountId, _new_coldkey: &AccountId) -> Result<(), ()> {
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type AccountId = u64;
    type Proxy = ProxyStub<AccountId>;
    type Commitments = CommitmentsStub<AccountId>;
    type Authorship = AuthorshipStub<AccountId>;
    type ColdkeySwap = ColdkeySwapStub<AccountId>;

    #[test]
    fn proxy_no_delegation() {
        let account: AccountId = 1;
        assert!(!Proxy::exists(&account));
        assert_eq!(Proxy::proxied(&account), None);
        assert_eq!(Proxy::real(account), account);
        assert!(!Proxy::is_pure(&account));
    }

    #[test]
    fn commitments_no_storage() {
        let account: AccountId = 1;
        assert!(Commitments::set_commitment(&account, b"data").is_ok());
        assert_eq!(Commitments::get_commitment(&account), None);
        assert_eq!(Commitments::rate_limit(), 0);
    }

    #[test]
    fn authorship_no_author() {
        assert_eq!(Authorship::author(), None::<AccountId>);
        assert!(Authorship::uncles().is_empty());
        // set_author is no-op, so nothing to verify
    }

    #[test]
    fn coldkey_swap_not_allowed() {
        let old: AccountId = 1;
        let new: AccountId = 2;
        assert!(!ColdkeySwap::is_coldkey_swap(&old));
        assert!(!ColdkeySwap::is_hotkey_schedule_swap(&old));
        assert_eq!(ColdkeySwap::coldkey_swap_block(&old), None);
        assert!(ColdkeySwap::do_coldkey_swap(&old, &new).is_err());
    }
}
