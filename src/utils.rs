use compio_log::info;
use std::sync::LazyLock;
use std::thread;

use gdt_cpus::{CpuInfo, Lp, pin_thread_to_core};

pub static LOGICAL_CORES: LazyLock<Vec<Lp>> =
    LazyLock::new(|| CpuInfo::detect().expect("Failed to detect CPU info").lps);
static BEST_CORE: LazyLock<Lp> = LazyLock::new(|| {
    let lp = LOGICAL_CORES
        .iter()
        .max_by_key(|lp| lp.perf_hint)
        .expect("Empty logicial processors");
    lp.clone()
});

/// Pin the current thread to the most performant logical processor.
///
/// On Linux, this is ignored because BEST_CORE becomes unreliable.
/// See https://github.com/gdt-tools/gdt-cpus-rs/issues/15
pub fn pin_to_best_core() -> color_eyre::Result<()> {
    #[cfg(not(target_os = "linux"))]
    pin_to_core(BEST_CORE.os_id as _)?;
    Ok(())
}

/// Pin the current thread to the selected logical processor.
pub fn pin_to_core(os_id: u16) -> color_eyre::Result<()> {
    let rust_tid = thread::current().id().as_u64();

    info!("Pinning Thread 0x{rust_tid:04x} to CPU {}", os_id);
    pin_thread_to_core(os_id as _)?;

    Ok(())
}
