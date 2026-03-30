use frame_system::CheckNonce as FrameCheckNonce;

/// Temporary runtime-local alias while the chain restart rebuilds its
/// transaction extension surface.
pub type CheckNonce<T> = FrameCheckNonce<T>;
