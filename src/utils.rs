use std::random::{Rng, SystemRng};
use std::sync::LazyLock;
use std::thread;
use std::time::{Duration, Instant};

use compio_log::info;
#[cfg(any(
        target_os = "windows",
        target_os = "linux",
        target_os = "android",
        // macos: not available
    ))]
use gdt_cpus::pin_thread_to_core;
use gdt_cpus::{CpuInfo, Lp};
use sha2::Digest;

pub static LOGICAL_CORES: LazyLock<Vec<Lp>> =
    LazyLock::new(|| CpuInfo::detect().expect("Failed to detect CPU info").lps);
static BEST_CORE: LazyLock<Lp> = LazyLock::new(|| {
    let lp = LOGICAL_CORES
        .iter()
        .max_by_key(|lp| lp.perf_hint)
        .expect("Empty logicial processors");
    lp.clone()
});

const CHUNK_SIZE: usize = 16 * 1024; // 16 KiB

/// Runs SHA-256 hashing until the specified duration has elapsed.
///
/// Returns the amount of data processed during the period in MiB.
pub fn sha256_workload(dur: Duration) -> usize {
    let mut payload = [0_u8; CHUNK_SIZE];
    SystemRng.fill_bytes(&mut payload);

    let calc_times = 0x100000 / CHUNK_SIZE;
    let mut mib_count = 0;

    let start_instant = Instant::now();
    loop {
        if start_instant.elapsed() >= dur {
            break mib_count;
        }

        let mut hasher = sha2::Sha256::new();
        for _ in 0..calc_times {
            hasher.update(&payload);
        }
        let _ = std::hint::black_box(hasher.finalize());

        mib_count += 1;
    }
}

/// Pin the current thread to the most performant logical processor.
pub fn pin_to_best_core() -> color_eyre::Result<()> {
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
