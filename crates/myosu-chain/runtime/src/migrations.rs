//! Runtime-local migration entry point restored for the chain restart.
//!
//! The active runtime file still references historical migration surfaces.
//! Keeping this module present lets the compile frontier move from missing
//! files to the actual dependency and runtime wiring work.
