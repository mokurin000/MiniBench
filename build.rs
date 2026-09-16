use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let is_android_arm64 = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android")
        && std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("aarch64");

    if is_android_arm64 || true {
        println!("cargo:rerun-if-changed=pikafish/pikafish-android-arm64");
        println!("cargo:rerun-if-changed=pikafish/pikafish.nnue");

        let bundled = PathBuf::from(std::env::var("OUT_DIR")?);
        std::fs::write(
            bundled.join("pikafish"),
            include_bytes!("pikafish/pikafish-android-arm64"),
        )?;
        std::fs::write(
            bundled.join("pikafish.nnue"),
            include_bytes!("pikafish/pikafish.nnue"),
        )?;
    }

    Ok(())
}
