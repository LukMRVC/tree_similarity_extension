//! Multi-threaded entry filtering for one batch of raw index entries.
//!
//! Pure Rust: nothing here may call into Postgres, since it runs on threads
//! other than the backend's.

use std::sync::atomic::{AtomicUsize, Ordering};

use super::storage::{decode_entry, ChunkSource, StreamReader};
use crate::pipelines::Lb;
use crate::types::UnifiedTreeIndex;

/// Entries a thread claims at a time; small enough to balance uneven trees.
const SLICE: usize = 64;

/// A batch of encoded entries: entry `i` is `buf[bounds[i]..bounds[i + 1]]`.
pub struct Batch {
    pub buf: Vec<u8>,
    pub bounds: Vec<usize>,
}

impl Default for Batch {
    fn default() -> Self {
        Self {
            buf: Vec::new(),
            bounds: vec![0],
        }
    }
}

impl Batch {
    pub fn len(&self) -> usize {
        self.bounds.len() - 1
    }

    /// Replace the contents with the next entries from `reader`, stopping once
    /// at least `max_bytes` are buffered. False when the stream is exhausted.
    pub fn refill<S: ChunkSource>(&mut self, reader: &mut StreamReader<S>, max_bytes: usize) -> bool {
        self.buf.clear();
        self.bounds.truncate(1);
        while self.buf.len() < max_bytes && reader.next_raw_entry(&mut self.buf) {
            self.bounds.push(self.buf.len());
        }
        self.len() > 0
    }
}

/// Whether `tree` is within `k` of every query.
pub fn matches(queries: &[(UnifiedTreeIndex, i32)], lb: Lb, tree: &UnifiedTreeIndex) -> bool {
    queries.iter().all(|(q, k)| lb.within(q, tree, *k) <= *k)
}

/// TIDs of the entries in `batch` within every `(query, k)`, using `threads`
/// threads (the caller's included). `Err` carries a worker's panic message.
pub fn filter_batch(
    batch: &Batch,
    queries: &[(UnifiedTreeIndex, i32)],
    lb: Lb,
    threads: usize,
) -> Result<Vec<u64>, String> {
    let cursor = AtomicUsize::new(0);
    let work = || {
        let mut out = Vec::new();
        loop {
            let start = cursor.fetch_add(SLICE, Ordering::Relaxed);
            if start >= batch.len() {
                return out;
            }
            for i in start..(start + SLICE).min(batch.len()) {
                let (tid, tree) = decode_entry(&batch.buf[batch.bounds[i]..batch.bounds[i + 1]]);
                if matches(queries, lb, &tree) {
                    out.push(tid);
                }
            }
        }
    };

    std::thread::scope(|s| {
        let workers: Vec<_> = (1..threads).map(|_| s.spawn(work)).collect();
        let mut tids = work();
        for w in workers {
            tids.extend(w.join().map_err(panic_message)?);
        }
        Ok(tids)
    })
}

fn panic_message(e: Box<dyn std::any::Any + Send>) -> String {
    e.downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| e.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iam::storage::StreamWriter;
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

    fn reader(trees: &[UnifiedTreeIndex]) -> StreamReader<std::vec::IntoIter<Vec<u8>>> {
        let mut w = StreamWriter::new(Vec::<Vec<u8>>::new());
        for (i, t) in trees.iter().enumerate() {
            w.push(i as u64, t);
        }
        StreamReader::new(w.finish().0.into_iter())
    }

    /// Small batches split the stream at entry boundaries without losing any.
    #[test]
    fn refill_covers_every_entry_once() {
        let trees: Vec<_> = (0..400).map(|i| uti(&tree(i))).collect();
        for max_bytes in [1, 100, 5000, usize::MAX] {
            let mut r = reader(&trees);
            let mut batch = Batch::default();
            let mut tids = Vec::new();
            let mut n_batches = 0;
            while batch.refill(&mut r, max_bytes) {
                n_batches += 1;
                for i in 0..batch.len() {
                    tids.push(decode_entry(&batch.buf[batch.bounds[i]..batch.bounds[i + 1]]).0);
                }
            }
            assert_eq!(tids, (0..400).collect::<Vec<u64>>(), "max_bytes={max_bytes}");
            assert!(max_bytes != 1 || n_batches == 400);
            assert!(max_bytes != usize::MAX || n_batches == 1);
        }
    }

    #[test]
    fn threads_match_serial_for_every_lb() {
        let trees: Vec<_> = (0..400).map(|i| uti(&tree(i))).collect();
        let mut batch = Batch::default();
        assert!(batch.refill(&mut reader(&trees), usize::MAX));
        for lb in Lb::ALL {
            for (qi, k) in [(2, 1), (7, 2), (13, 0), (3, 3)] {
                let queries = [(trees[qi].clone(), k)];
                let want: Vec<u64> = (0..trees.len())
                    .filter(|&i| lb.within(&queries[0].0, &trees[i], k) <= k)
                    .map(|i| i as u64)
                    .collect();
                assert!(!want.is_empty() && want.len() < trees.len());
                for threads in [1, 3, 8] {
                    let mut got = filter_batch(&batch, &queries, lb, threads).unwrap();
                    got.sort_unstable();
                    assert_eq!(got, want, "lb={} q={qi} k={k} threads={threads}", lb.name());
                }
            }
        }
    }

    #[test]
    fn empty_batch() {
        let batch = Batch::default();
        assert!(filter_batch(&batch, &[(uti("{a}"), 1)], Lb::SedStruct, 4).unwrap().is_empty());
    }
}
