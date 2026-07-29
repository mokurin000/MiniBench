//! Application entry point.

#![windows_subsystem = "windows"]

use main::Result;

/// Desktop entry point (Windows / Linux / macOS).
#[cfg(not(target_os = "android"))]
fn main() -> Result<()> {
    use tracing_subscriber::EnvFilter;
    use winio::prelude::*;

    use main::APP_ID;
    use main::model::MainModel;

    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    App::builder()
        .name(APP_ID)
        .build()?
        .block_on(MainModel::run_until_event(()))
}

/// Android entry point is `android_main` instead.
#[cfg(target_os = "android")]
fn main() -> Result<()> {
    unreachable!("Android entry point is `android_main` in `android.rs`")
}
