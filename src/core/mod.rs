//! The always-on build pipeline — no cargo features required.
//!
//! Groups content loading + parsing ([`content`]), navigation/context
//! computation ([`nav`]), shared [`config`], and the [`build`] orchestrator.
//! Public types are re-exported from the crate root (`src/lib.rs`); this module
//! tree is the internal organisation behind that facade.

pub mod build;
pub mod config;
pub mod content;
pub mod nav;
