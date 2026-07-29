use std::sync::LazyLock;
use std::thread;

use compio_log::info;
use gdt_cpus::{AffinityMask, CpuInfo, set_thread_affinity, set_thread_priority};

static BEST_CORE: LazyLock<AffinityMask> = LazyLock::new(|| {
    let lps = CpuInfo::detect().expect("Failed to detect CPU info").lps;
    let lp = lps
        .into_iter()
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
