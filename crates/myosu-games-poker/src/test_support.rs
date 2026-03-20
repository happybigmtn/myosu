use rbp_cards::{Isomorphism, IsomorphismIterator, Street};
use rbp_gameplay::Abstraction;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::OnceLock;

#[derive(Serialize, Deserialize)]
struct CompatEncoder(BTreeMap<Isomorphism, Abstraction>);

pub(crate) fn root_encoder_bytes() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES
        .get_or_init(|| {
            let map = IsomorphismIterator::from(Street::Pref)
                .map(|isomorphism| (isomorphism, Abstraction::from((Street::Pref, 0))))
                .collect::<BTreeMap<Isomorphism, Abstraction>>();
            bincode::serialize(&CompatEncoder(map)).unwrap()
        })
        .as_slice()
}
