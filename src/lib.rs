//! Root module of the app.
#![feature(random)]

pub mod model;
pub mod utils;

/// Shorthand for `Result<T, color_eyre::Report>`.
pub type Result<T> = std::result::Result<T, color_eyre::Report>;
/// Unique ID for this application.
pub const APP_ID: &str = "io.github.mokurin000.minibench";

#[cfg(target_os = "android")]
mod android;
