//! Minimal checked arithmetic helpers for the Phase 1 restart slice.
//!
//! The full fixed-point epoch math returns in a later slice. For now we keep
//! the restart boundary small and replace the broken subtensor math surface
//! with local checked arithmetic helpers.

pub trait SafeDiv<Rhs = Self> {
    type Output;

    fn safe_div(self, rhs: Rhs) -> Option<Self::Output>;
}

impl SafeDiv for u64 {
    type Output = u64;

    fn safe_div(self, rhs: Self) -> Option<Self::Output> {
        if rhs == 0 {
            None
        } else {
            Some(self / rhs)
        }
    }
}

impl SafeDiv for i64 {
    type Output = i64;

    fn safe_div(self, rhs: Self) -> Option<Self::Output> {
        if rhs == 0 {
            None
        } else {
            Some(self / rhs)
        }
    }
}

pub fn checked_ln(_value: i64) -> Option<i64> {
    None
}

pub fn checked_exp(_value: i64) -> Option<i64> {
    None
}
