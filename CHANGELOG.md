# Changelog

Notable changes to this project. The crate is unversioned (`0.0.0`), so entries are listed per commit, newest
first. Format loosely follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## 57528cb — 2026-09-26

Persistent scan thread pool, pipelined batches, single-copy reads.

### Changed
- `tree_search_iam.scan_threads > 1` now uses a per-session pool of worker threads, started by the first threaded
  scan, instead of starting threads for every batch. Each whole batch goes to one worker.
- The backend reads ahead while workers check entries (up to 2 batches per worker queued). It still checks for
  interrupts at least every 10 ms, and a cancelled or failed scan never leaks results into the next one.
- Worker threads block all signals, so Postgres's signals (cancel, termination) reach the backend thread.
- Batches are 64 KiB (was 1 MiB).
- Index pages are copied once, straight from shared buffers into the reader's buffer (was three copies), and
  entries are decoded into a reused tree instead of allocating per entry.

### Performance
20 queries, release build, before → after:

| Dataset   | 1 thread         | best thread count          |
|-----------|------------------|----------------------------|
| rna       | 132 → 104 ms     | 8 threads: 250 → 77 ms     |
| ptb       | ~1720 ms (same)  | 16 threads: 363 → ~280 ms  |
| sentiment | 1420 → 1330 ms   | 16 threads: 445 → 176 ms   |
| treefam   | 1245 → 1165 ms   | 8 threads: 1690 → 615 ms   |

### Docs
- `docs/usage.md`: how `scan_threads` works and when it helps.
- README: I/O configuration section (`io_method`, `io_uring` build and benchmark setup); `cargo-pgrx` pinned to
  0.19.1.

## 683291e — 2026-09-26

Threaded scans, prefetched reads, bulk-read ring for large indexes.

### Added
- GUC `tree_search_iam.scan_threads` (default `1`): check index entries on several threads within one scan.
  Results are identical at any thread count.
- Index pages are read through a Postgres read stream, which prefetches ahead of the scan.
- An index larger than a quarter of `shared_buffers` is scanned through a small private buffer ring
  (`BAS_BULKREAD`), so it no longer evicts everything else; smaller indexes stay cached. VACUUM uses its own ring.
- Bench: `iam_<lb>@t<N>` methods run the index scan with `scan_threads = N`.
- `docs/usage.md`: usage and reference doc; README intro and quick start.

### Removed
- PostgreSQL 16 support. The read stream API needs PG17+.

### Known issues
- Threads were started for every batch, so more threads made small scans slower (fixed in 57528cb).
