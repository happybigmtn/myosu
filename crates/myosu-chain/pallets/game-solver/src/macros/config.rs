use alloc::vec::Vec;

pub trait ProxyInterface<AccountId> {
    fn exists(delegate: &AccountId) -> bool;
    fn proxied(who: &AccountId) -> Option<AccountId>;
    fn real(who: AccountId) -> AccountId;
    fn is_pure(who: &AccountId) -> bool;
}

pub trait CommitmentsInterface<AccountId> {
    fn set_commitment(who: &AccountId, data: &[u8]) -> Result<(), ()>;
    fn get_commitment(who: &AccountId) -> Option<Vec<u8>>;
    fn rate_limit() -> u64;
}

pub trait AuthorshipProvider<AccountId> {
    fn author() -> Option<AccountId>;
    fn uncles() -> Vec<AccountId>;
    fn set_author(author: &AccountId);
}
