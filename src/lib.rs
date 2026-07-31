//! Root module of the app.
#![feature(random, thread_id_value)]

pub mod model;
pub mod utils;
pub mod workload;

/// Shorthand for `Result<T, color_eyre::Report>`.
pub type Result<T> = std::result::Result<T, color_eyre::Report>;
/// Unique ID for this application.
pub const APP_ID: &str = "io.github.mokurin000.minibench";

#[cfg(target_os = "android")]
mod android;
