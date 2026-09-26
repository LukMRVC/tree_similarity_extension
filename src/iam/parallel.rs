//! Multi-threaded index scans: a per-backend pool of worker threads checks
//! batches of raw entries while the backend reads the next ones.
//!
//! Pure Rust: nothing here may call into Postgres, since the workers are not
//! the backend's thread. The backend's side (`run_scan`) reaches Postgres only
//! through the callbacks it is given.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::storage::{decode_entry_into, empty_tree, Batch, ChunkSource, StreamReader};
use crate::pipelines::Lb;
use crate::types::UnifiedTreeIndex;

/// How long the backend waits for a result before calling `poll` again.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Whether `tree` is within `k` of every query.
pub fn matches(queries: &[(UnifiedTreeIndex, i32)], lb: Lb, tree: &UnifiedTreeIndex) -> bool {
    queries.iter().all(|(q, k)| lb.within(q, tree, *k) <= *k)
}

/// Append the TIDs of the entries in `batch` that match to `out`. `tree` is
/// scratch space, reused across entries.
fn filter_batch(
    batch: &Batch,
    queries: &[(UnifiedTreeIndex, i32)],
    lb: Lb,
    tree: &mut UnifiedTreeIndex,
    out: &mut Vec<u64>,
) {
    for i in 0..batch.len() {
        let tid = decode_entry_into(batch.entry(i), tree);
        if matches(queries, lb, tree) {
            out.push(tid);
        }
    }
}

/// What every batch of one scan is checked against.
struct Scan {
    queries: Vec<(UnifiedTreeIndex, i32)>,
    lb: Lb,
    /// Set when the scan ends early (e.g. on an ERROR): its queued batches
    /// are then skipped.
    cancelled: AtomicBool,
}

struct Job {
    scan: Arc<Scan>,
    batch: Batch,
    done: Sender<Done>,
}

/// A checked batch, handed back so its buffer can be reused.
struct Done {
    batch: Batch,
    tids: Result<Vec<u64>, String>,
}

/// Worker threads, started on the first threaded scan and kept for the rest
/// of the backend's life. Dropping `jobs` lets the workers exit.
struct Pool {
    jobs: Sender<Job>,
    size: usize,
}

static POOL: Mutex<Option<Pool>> = Mutex::new(None);

/// The job queue of a pool with exactly `threads` workers, (re)starting it
/// when the size changed.
fn pool_jobs(threads: usize) -> Sender<Job> {
    let mut pool = POOL.lock().unwrap_or_else(|e| e.into_inner());
    if pool.as_ref().map(|p| p.size) != Some(threads) {
        *pool = Some(Pool::start(threads));
    }
    pool.as_ref().unwrap().jobs.clone()
}

impl Pool {
    fn start(size: usize) -> Self {
        let (jobs, queue) = mpsc::channel::<Job>();
        let queue = Arc::new(Mutex::new(queue));
        // Postgres handles its signals (cancel, termination, latches) on the
        // backend's thread; the kernel may deliver a process signal to any
        // thread that doesn't block it. Workers inherit this mask.
        with_signals_blocked(|| {
            for i in 0..size {
                let queue = Arc::clone(&queue);
                std::thread::Builder::new()
                    .name(format!("tree_search_iam worker {i}"))
                    .spawn(move || work(&queue))
                    .expect("tree_search_iam: could not start a scan thread");
            }
        });
        Self { jobs, size }
    }
}

fn with_signals_blocked(f: impl FnOnce()) {
    unsafe {
        let mut all: libc::sigset_t = std::mem::zeroed();
        let mut old: libc::sigset_t = std::mem::zeroed();
        libc::sigfillset(&mut all);
        libc::pthread_sigmask(libc::SIG_BLOCK, &all, &mut old);
        f();
        libc::pthread_sigmask(libc::SIG_SETMASK, &old, std::ptr::null_mut());
    }
}

/// A worker: check batches until the pool is dropped.
fn work(queue: &Mutex<Receiver<Job>>) {
    let mut tree = empty_tree();
    loop {
        let job = queue.lock().unwrap_or_else(|e| e.into_inner()).recv();
        let Ok(Job { scan, batch, done }) = job else {
            return;
        };
        if scan.cancelled.load(Ordering::Relaxed) {
            continue;
        }
        let tids = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut out = Vec::new();
            filter_batch(&batch, &scan.queries, scan.lb, &mut tree, &mut out);
            out
        }))
        .map_err(panic_message);
        // The scan may be gone already; its result is then not needed.
        let _ = done.send(Done { batch, tids });
    }
}

fn panic_message(e: Box<dyn std::any::Any + Send>) -> String {
    e.downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| e.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".into())
}

/// Marks a scan cancelled when the backend leaves `run_scan`, however it leaves.
struct CancelOnDrop(Arc<Scan>);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancelled.store(true, Ordering::Relaxed);
    }
}

/// Check every entry of `reader` on `threads` pool workers. The calling
/// (backend) thread reads batches of about `batch_bytes` and keeps up to two
/// per worker queued, so reading overlaps with checking. Matching TIDs go to
/// `on_tids`, in no particular order; `poll` runs after every batch read and
/// at least every `POLL_INTERVAL` while waiting (for interrupt checks).
/// Either callback may unwind; the scan's queued batches are then dropped.
/// `Err` carries a worker's panic message.
pub fn run_scan<S: ChunkSource>(
    reader: &mut StreamReader<S>,
    queries: Vec<(UnifiedTreeIndex, i32)>,
    lb: Lb,
    threads: usize,
    batch_bytes: usize,
    mut on_tids: impl FnMut(&[u64]),
    mut poll: impl FnMut(),
) -> Result<(), String> {
    let scan = Arc::new(Scan {
        queries,
        lb,
        cancelled: AtomicBool::new(false),
    });
    let _cancel = CancelOnDrop(Arc::clone(&scan));
    let jobs = pool_jobs(threads);
    let (done_tx, done_rx) = mpsc::channel();
    let max_in_flight = 2 * threads;
    let mut spare: Vec<Batch> = Vec::new();
    let mut in_flight = 0;
    let mut exhausted = false;
    loop {
        while !exhausted && in_flight < max_in_flight {
            let mut batch = spare.pop().unwrap_or_default();
            if !reader.next_batch(batch_bytes, &mut batch) {
                exhausted = true;
                break;
            }
            let job = Job {
                scan: Arc::clone(&scan),
                batch,
                done: done_tx.clone(),
            };
            jobs.send(job).expect("tree_search_iam: scan threads are gone");
            in_flight += 1;
            poll();
        }
        if in_flight == 0 {
            return Ok(());
        }
        match done_rx.recv_timeout(POLL_INTERVAL) {
            Ok(done) => {
                in_flight -= 1;
                on_tids(&done.tids?);
                spare.push(done.batch);
            }
            Err(RecvTimeoutError::Timeout) => poll(),
            Err(RecvTimeoutError::Disconnected) => unreachable!("done_tx is still held"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iam::storage::tests::reader;
    use std::ffi::CString;

    fn uti(s: &str) -> UnifiedTreeIndex {
        UnifiedTreeIndex::parse(CString::new(s).unwrap().as_c_str()).unwrap()
    }

    /// Small trees over a tiny alphabet, so many pairs are within small k.
    fn tree(i: usize) -> String {
        let l = |j: usize| ["a", "b", "c"][(i / 3usize.pow(j as u32)) % 3];
        match i % 4 {
            0 => format!("{{{}}}", l(0)),
            1 => format!("{{{}{{{}}}}}", l(0), l(1)),
            2 => format!("{{{}{{{}}}{{{}}}}}", l(0), l(1), l(2)),
            _ => format!("{{{}{{{}{{{}}}}}{{{}}}}}", l(0), l(1), l(2), l(3)),
        }
    }

    fn scan(
        trees: &[UnifiedTreeIndex],
        queries: &[(UnifiedTreeIndex, i32)],
        lb: Lb,
        threads: usize,
        batch_bytes: usize,
    ) -> Vec<u64> {
        let mut got = Vec::new();
        let on_tids = |t: &[u64]| got.extend_from_slice(t);
        run_scan(&mut reader(trees), queries.to_vec(), lb, threads, batch_bytes, on_tids, || {}).unwrap();
        got.sort_unstable();
        got
    }

    #[test]
    fn pool_matches_serial_for_every_lb() {
        let trees: Vec<_> = (0..400).map(|i| uti(&tree(i))).collect();
        for lb in Lb::ALL {
            for (qi, k) in [(2, 1), (7, 2), (13, 0), (3, 3)] {
                let queries = [(trees[qi].clone(), k)];
                let want: Vec<u64> = (0..trees.len())
                    .filter(|&i| matches(&queries, lb, &trees[i]))
                    .map(|i| i as u64)
                    .collect();
                assert!(!want.is_empty() && want.len() < trees.len());
                // Changing sizes also restarts the pool between scans.
                for (threads, batch_bytes) in [(1, 1), (3, 100), (8, 1000), (3, usize::MAX)] {
                    let got = scan(&trees, &queries, lb, threads, batch_bytes);
                    assert_eq!(got, want, "lb={} q={qi} k={k} threads={threads}", lb.name());
                }
            }
        }
    }

    #[test]
    fn empty_scan() {
        assert!(scan(&[], &[(uti("{a}"), 1)], Lb::SedStruct, 4, 100).is_empty());
    }

    /// A scan abandoned by an unwinding callback (an ERROR in the backend)
    /// leaves the pool usable, and its leftover results don't leak into the
    /// next scan.
    #[test]
    fn abandoned_scan_leaves_pool_usable() {
        let trees: Vec<_> = (0..400).map(|i| uti(&tree(i))).collect();
        let queries = [(trees[7].clone(), 2)];
        let want = scan(&trees, &queries, Lb::SedStruct, 4, 100);
        let mut polls = 0;
        let abandoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let poll = || {
                polls += 1;
                if polls == 5 {
                    panic!("canceled");
                }
            };
            run_scan(&mut reader(&trees), queries.to_vec(), Lb::SedStruct, 4, 100, |_| {}, poll)
        }));
        assert!(abandoned.is_err());
        assert_eq!(scan(&trees, &queries, Lb::SedStruct, 4, 100), want);
    }
}
