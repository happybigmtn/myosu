use codec::{Decode, DecodeWithMemTracking, Encode};
use core::marker::PhantomData;
use scale_info::TypeInfo;

/// Temporary stand-in for the historical transaction payment wrapper.
#[derive(Clone, Debug, Decode, DecodeWithMemTracking, Encode, Eq, PartialEq, TypeInfo)]
pub struct ChargeTransactionPaymentWrapper<T> {
    _marker: PhantomData<T>,
}

impl<T> ChargeTransactionPaymentWrapper<T> {
    pub fn new<U>(_inner: U) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
