//! Shared-buffer allocation diagnostics moved out of host.rs to respect the source size limit.

use std::sync::{Mutex, OnceLock};

/// Which lines of ours make shared buffers ([`super::Host::shared_buffer`]), how
/// many and how many bytes: each is memory of its own that is waited for
/// until it is mapped ([`super::mapped`]), and a run made 864,839 of them in ten
/// minutes (10 GB, 12 KB each). The busiest lines are logged every 20,000.
#[cfg(feature = "diagnostics")]
pub(super) fn shared_buffer_caller(at: &'static std::panic::Location<'static>, size: u64) {
    static TRACE: OnceLock<bool> = OnceLock::new();
    if !*TRACE.get_or_init(|| std::env::var_os("VICTORY_SHARED_BUFFER_TRACE").is_some_and(|v| v == "1")) {
        return;
    }
    static BY: Mutex<Option<std::collections::HashMap<(&'static str, u32), (u64, u64)>>> = Mutex::new(None);
    let mut by = BY.lock().unwrap_or_else(|e| e.into_inner());
    let map = by.get_or_insert_with(Default::default);
    let entry = map.entry((at.file(), at.line())).or_default();
    entry.0 += 1;
    entry.1 += size;
    let n: u64 = map.values().map(|v| v.0).sum();
    if n.is_multiple_of(20_000) {
        let mut top: Vec<_> = map.iter().map(|(k, v)| (*k, *v)).collect();
        top.sort_by_key(|t| std::cmp::Reverse(t.1.0));
        let said: Vec<String> = top
            .iter()
            .take(8)
            .map(|((file, line), (count, bytes))| format!("{file}:{line} {count} ({} MB)", bytes >> 20))
            .collect();
        tracing::info!("shared buffers made so far by line ({n} in all): {}", said.join(", "));
    }
}

