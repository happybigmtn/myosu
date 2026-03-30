#![cfg_attr(not(feature = "std"), no_std)]

#[macro_export]
macro_rules! prod_or_fast {
    ($prod:expr, $fast:expr) => {{
        #[cfg(feature = "fast-blocks")]
        {
            $fast
        }
        #[cfg(not(feature = "fast-blocks"))]
        {
            $prod
        }
    }};
}
