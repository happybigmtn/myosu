extern crate alloc;

use alloc::{vec, vec::Vec};

use sp_genesis_builder::PresetId;

pub const DEVELOPMENT_PRESET: &str = "development";

pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
    if id.as_ref() == DEVELOPMENT_PRESET.as_bytes() {
        Some(b"{}".to_vec())
    } else {
        None
    }
}

pub fn preset_names() -> Vec<PresetId> {
    vec![PresetId::from(DEVELOPMENT_PRESET)]
}
