use codec::{Decode, DecodeWithMemTracking, Encode};
use core::marker::PhantomData;
use scale_info::TypeInfo;

/// Temporary stand-in for the historical sudo transaction wrapper.
#[derive(
    Clone, Copy, Debug, Decode, DecodeWithMemTracking, Default, Encode, Eq, PartialEq, TypeInfo,
)]
pub struct SudoTransactionExtension<T>(PhantomData<T>);

impl<T> SudoTransactionExtension<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}
