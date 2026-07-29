use std::sync::LazyLock;
use std::thread;

use compio_log::info;
use gdt_cpus::{AffinityMask, CpuInfo, Lp, set_thread_affinity, set_thread_priority};

static LOGICAL_CORES: LazyLock<Vec<Lp>> =
    LazyLock::new(|| CpuInfo::detect().expect("Failed to detect CPU info").lps);
static BEST_CORE: LazyLock<AffinityMask> = LazyLock::new(|| {
    let lp = LOGICAL_CORES
        .iter()
        .max_by_key(|lp| lp.perf_hint)
        .expect("Empty logicial processors");
    AffinityMask::from_cores(&[lp.core as _])
});

pub fn boost_current_thread() -> color_eyre::Result<()> {
    let rust_tid = thread::current().id();

    info!("Boosting {rust_tid:?}");

    #[cfg(any(
        target_os = "windows",
        target_os = "linux",
        target_os = "android",
        // macos: not available
    ))]
    set_thread_affinity(&BEST_CORE)?;
    _ = set_thread_priority(gdt_cpus::ThreadPriority::Highest)?;

    Ok(())
}
