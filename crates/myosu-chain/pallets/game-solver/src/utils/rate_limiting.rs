use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

use crate::NetUid;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum RateLimitKey {
    LastTransaction,
    DelegateTake,
    ChildKeyTake,
    NetworkLastRegistered,
    AddStake(NetUid),
    RemoveStake(NetUid),
    ServeAxon(NetUid),
    ServePrometheus(NetUid),
    SetWeights(NetUid),
    RegisterNeuron(NetUid),
}
