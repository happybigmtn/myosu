#![cfg_attr(not(test), deny(unsafe_code))]

//! Workspace anchor for the `chain:runtime` restart lane.
//!
//! The runtime, node, and common crates are introduced as sibling manifests in
//! this slice, then admitted to the active build graph in later approved slices.
