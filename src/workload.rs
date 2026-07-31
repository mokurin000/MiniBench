use std::random::{Rng, SystemRng};
use std::time::{Duration, Instant};

use sha2::Digest as _;

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
