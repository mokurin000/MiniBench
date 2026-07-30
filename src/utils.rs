use std::random::{Rng, SystemRng};
use std::sync::LazyLock;
use std::thread;
use std::time::{Duration, Instant};

use compio_log::info;

use gdt_cpus::{CpuInfo, Lp, pin_thread_to_core};
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

/// Runs SHA-256 hashing until the specified duration has elapsed.
///
/// Returns the amount of data processed during the period in MiB,
/// and the period took exactly.
pub fn sha256_workload(dur: Duration) -> (usize, Duration) {
    let mut payload = vec![0_u8; 0x100000];
    SystemRng.fill_bytes(&mut payload);

    let mut processed_mib = 0;

    let start_instant = Instant::now();
    loop {
        let elapsed = start_instant.elapsed();
        if elapsed >= dur {
            break (processed_mib, elapsed);
        }

        let mut hasher = sha2::Sha256::new();
        hasher.update(&payload);
        let _ = std::hint::black_box(hasher.finalize());

        processed_mib += 1;
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
