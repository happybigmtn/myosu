use substrate_fixed::types::{I32F32 as FixedI32F32, I64F64 as FixedI64F64};

pub use substrate_fixed::types::{I32F32, I64F64};

pub trait SafeDiv: Sized {
    fn safe_div(self, rhs: Self) -> Self;
}

impl SafeDiv for FixedI32F32 {
    fn safe_div(self, rhs: Self) -> Self {
        if rhs == FixedI32F32::from_num(0) {
            FixedI32F32::from_num(0)
        } else {
            self / rhs
        }
    }
}

impl SafeDiv for FixedI64F64 {
    fn safe_div(self, rhs: Self) -> Self {
        if rhs == FixedI64F64::from_num(0) {
            FixedI64F64::from_num(0)
        } else {
            self / rhs
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{I32F32, I64F64, SafeDiv};

    #[test]
    fn zero_divisor_returns_zero_for_i32f32() {
        assert_eq!(I32F32::from_num(5).safe_div(I32F32::from_num(0)), I32F32::from_num(0));
    }

    #[test]
    fn zero_divisor_returns_zero_for_i64f64() {
        assert_eq!(I64F64::from_num(5).safe_div(I64F64::from_num(0)), I64F64::from_num(0));
    }
}
