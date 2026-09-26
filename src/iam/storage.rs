//! On-disk layout of a `tree_search_iam` index. Nothing here is WAL-logged.
//!
//! Block 0 is the metapage. Blocks 1.. hold one byte stream of entries, cut
//! into page-sized chunks; an entry may cross page boundaries. Each data page
//! stores a little-endian `u16` chunk length followed by the chunk bytes.
//!
//! Entry: `tid: u64 | n: u32 | labels: n × u64 | sizes: n × i32`.

use pgrx::pg_sys;

use crate::types::UnifiedTreeIndex;

pub const META_BLOCK: pg_sys::BlockNumber = 0;
const META_MAGIC: u32 = 0x7472_6565; // "tree"
const META_VERSION: u32 = 1;

/// Usable bytes per page after the page header, minus the chunk length prefix.
pub const CHUNK_CAP: usize = pg_sys::BLCKSZ as usize - page_header_size() - 2;

const fn page_header_size() -> usize {
    // MAXALIGN(SizeOfPageHeaderData); the header is 24 bytes on every platform.
    24
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetaPage {
    pub magic: u32,
    pub version: u32,
    /// Non-zero once a row was added after the build; scans refuse to run.
    pub stale: u32,
    pub n_entries: u64,
    pub n_data_pages: u32,
}

impl MetaPage {
    pub fn new(n_entries: u64, n_data_pages: u32) -> Self {
        Self {
            magic: META_MAGIC,
            version: META_VERSION,
            stale: 0,
            n_entries,
            n_data_pages,
        }
    }
}

// ============================================================================
// Pure encoding: entries <-> a stream of chunks
// ============================================================================

pub fn encode_entry(out: &mut Vec<u8>, tid: u64, tree: &UnifiedTreeIndex) {
    out.extend_from_slice(&tid.to_le_bytes());
    out.extend_from_slice(&(tree.tree_size as u32).to_le_bytes());
    for label in &tree.labels {
        out.extend_from_slice(&label.to_le_bytes());
    }
    for size in &tree.sizes {
        out.extend_from_slice(&size.to_le_bytes());
    }
}

/// Where finished chunks go (index pages, or a `Vec` in tests).
pub trait ChunkSink {
    fn write_chunk(&mut self, chunk: &[u8]);
}

/// Where chunks come from, in order. `None` once the stream is exhausted.
pub trait ChunkSource {
    fn next_chunk(&mut self) -> Option<Vec<u8>>;
}

/// Buffers encoded entries and hands them to the sink in `CHUNK_CAP` pieces.
pub struct StreamWriter<S: ChunkSink> {
    sink: S,
    buf: Vec<u8>,
    pub n_entries: u64,
}

impl<S: ChunkSink> StreamWriter<S> {
    pub fn new(sink: S) -> Self {
        Self {
            sink,
            buf: Vec::with_capacity(2 * CHUNK_CAP),
            n_entries: 0,
        }
    }

    pub fn push(&mut self, tid: u64, tree: &UnifiedTreeIndex) {
        encode_entry(&mut self.buf, tid, tree);
        self.n_entries += 1;
        let full = self.buf.len() / CHUNK_CAP * CHUNK_CAP;
        for chunk in self.buf[..full].chunks(CHUNK_CAP) {
            self.sink.write_chunk(chunk);
        }
        self.buf.drain(..full);
    }

    pub fn finish(mut self) -> (S, u64) {
        if !self.buf.is_empty() {
            self.sink.write_chunk(&self.buf);
        }
        (self.sink, self.n_entries)
    }
}

/// Decodes entries from a chunk source, pulling chunks as needed.
pub struct StreamReader<S: ChunkSource> {
    source: S,
    buf: Vec<u8>,
    pos: usize,
}

impl<S: ChunkSource> StreamReader<S> {
    pub fn new(source: S) -> Self {
        Self {
            source,
            buf: Vec::new(),
            pos: 0,
        }
    }

    /// Make sure `n` unread bytes are buffered; false if the stream ends first.
    fn fill(&mut self, n: usize) -> bool {
        while self.buf.len() - self.pos < n {
            let Some(chunk) = self.source.next_chunk() else {
                return false;
            };
            self.buf.drain(..self.pos);
            self.pos = 0;
            self.buf.extend_from_slice(&chunk);
        }
        true
    }

    fn take<const N: usize>(&mut self) -> [u8; N] {
        let bytes = self.buf[self.pos..self.pos + N].try_into().unwrap();
        self.pos += N;
        bytes
    }

    /// Buffer the whole next entry and return its byte length; `None` at the
    /// end of the stream.
    fn entry_len(&mut self) -> Option<usize> {
        if !self.fill(12) {
            return None;
        }
        let n = u32::from_le_bytes(self.buf[self.pos + 8..self.pos + 12].try_into().unwrap()) as usize;
        let len = 12 + n * 12;
        if !self.fill(len) {
            panic!("tree_search_iam: truncated index entry");
        }
        Some(len)
    }

    /// Only the heap TIDs, skipping the tree payloads.
    pub fn next_tid(&mut self) -> Option<u64> {
        let len = self.entry_len()?;
        let tid = u64::from_le_bytes(self.take());
        self.pos += len - 8;
        Some(tid)
    }

    pub fn next_entry(&mut self) -> Option<(u64, UnifiedTreeIndex)> {
        let len = self.entry_len()?;
        let entry = decode_entry(&self.buf[self.pos..self.pos + len]);
        self.pos += len;
        Some(entry)
    }

    /// Append the next entry's encoded bytes to `out`, undecoded.
    pub fn next_raw_entry(&mut self, out: &mut Vec<u8>) -> bool {
        let Some(len) = self.entry_len() else {
            return false;
        };
        out.extend_from_slice(&self.buf[self.pos..self.pos + len]);
        self.pos += len;
        true
    }
}

/// Decode one entry as written by `encode_entry`; `bytes` holds exactly that entry.
pub fn decode_entry(bytes: &[u8]) -> (u64, UnifiedTreeIndex) {
    let tid = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
    let n = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let (labels, sizes) = bytes[12..].split_at(n * 8);
    let labels = labels
        .chunks_exact(8)
        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
        .collect();
    let sizes = sizes
        .chunks_exact(4)
        .map(|b| i32::from_le_bytes(b.try_into().unwrap()))
        .collect();
    (
        tid,
        UnifiedTreeIndex {
            labels,
            sizes,
            tree_size: n,
        },
    )
}

// ============================================================================
// Buffer-manager layer
// ============================================================================

unsafe fn contents(page: pg_sys::Page) -> *mut u8 {
    unsafe { pg_sys::PageGetContents(page) as *mut u8 }
}

/// Append a new, exclusively locked, initialized page to the fork.
unsafe fn new_page(index: pg_sys::Relation, fork: pg_sys::ForkNumber::Type) -> pg_sys::Buffer {
    unsafe {
        let bmr = pg_sys::BufferManagerRelation {
            rel: index,
            smgr: std::ptr::null_mut(),
            relpersistence: 0,
        };
        let buf = pg_sys::ExtendBufferedRel(
            bmr,
            fork,
            std::ptr::null_mut(),
            pg_sys::ExtendBufferedFlags::EB_LOCK_FIRST,
        );
        let page = pg_sys::BufferGetPage(buf);
        pg_sys::PageInit(page, pg_sys::BLCKSZ as usize, 0);
        buf
    }
}

unsafe fn read_page(
    index: pg_sys::Relation,
    block: pg_sys::BlockNumber,
    lock: u32,
) -> pg_sys::Buffer {
    unsafe {
        let buf = pg_sys::ReadBufferExtended(
            index,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            block,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            std::ptr::null_mut(),
        );
        pg_sys::LockBuffer(buf, lock as i32);
        buf
    }
}

/// Write the chunk into a freshly initialized, locked page and release it.
unsafe fn fill_and_release(buf: pg_sys::Buffer, chunk: &[u8]) {
    unsafe {
        let page = pg_sys::BufferGetPage(buf);
        let dst = contents(page);
        let len = chunk.len() as u16;
        std::ptr::copy_nonoverlapping(len.to_le_bytes().as_ptr(), dst, 2);
        std::ptr::copy_nonoverlapping(chunk.as_ptr(), dst.add(2), chunk.len());
        let header = page as *mut pg_sys::PageHeaderData;
        (*header).pd_lower = (page_header_size() + 2 + chunk.len()) as u16;
        pg_sys::MarkBufferDirty(buf);
        pg_sys::UnlockReleaseBuffer(buf);
    }
}

/// Appends each chunk as a new data page of the index.
pub struct PageSink {
    index: pg_sys::Relation,
    pub n_pages: u32,
}

impl PageSink {
    pub fn new(index: pg_sys::Relation) -> Self {
        Self { index, n_pages: 0 }
    }
}

impl ChunkSink for PageSink {
    fn write_chunk(&mut self, chunk: &[u8]) {
        unsafe {
            let buf = new_page(self.index, pg_sys::ForkNumber::MAIN_FORKNUM);
            fill_and_release(buf, chunk);
        }
        self.n_pages += 1;
    }
}

/// Reads the data pages 1..=n_pages in order through a read stream, so
/// Postgres fetches upcoming pages while the current ones are processed.
pub struct PageSource {
    stream: *mut pg_sys::ReadStream,
    /// Read by the stream's callback through a raw pointer; boxed so that
    /// address stays fixed, and kept here so it lives as long as the stream.
    blocks: Box<BlockRange>,
    /// A ring this source allocated and must free; null when the caller owns it.
    own_strategy: pg_sys::BufferAccessStrategy,
}

struct BlockRange {
    next: pg_sys::BlockNumber,
    last: pg_sys::BlockNumber,
}

/// True when an index of `n_data_pages` should be read through a ring, given
/// `n_buffers` shared buffers.
fn use_bulkread(n_data_pages: u32, n_buffers: i32) -> bool {
    n_data_pages as i64 > n_buffers as i64 / 4
}

/// Read-stream callback: hands out blocks `next..=last`, then "no more".
unsafe extern "C-unwind" fn next_block(
    _stream: *mut pg_sys::ReadStream,
    private: *mut std::ffi::c_void,
    _per_buffer_data: *mut std::ffi::c_void,
) -> pg_sys::BlockNumber {
    let range = unsafe { &mut *(private as *mut BlockRange) };
    if range.next > range.last {
        return pg_sys::InvalidBlockNumber;
    }
    range.next += 1;
    range.next - 1
}

impl PageSource {
    /// For index scans. An index larger than a quarter of shared buffers is
    /// read through a small private ring of buffers (`BAS_BULKREAD`) so one
    /// scan can't evict everything else; a smaller one goes through shared
    /// buffers normally and stays cached. Postgres sizes seq scans the same way.
    pub fn for_scan(index: pg_sys::Relation, meta: &MetaPage) -> Self {
        if !use_bulkread(meta.n_data_pages, unsafe { pg_sys::NBuffers }) {
            return Self::open(index, meta, std::ptr::null_mut(), false);
        }
        let ring = unsafe { pg_sys::GetAccessStrategy(pg_sys::BufferAccessStrategyType::BAS_BULKREAD) };
        Self::open(index, meta, ring, true)
    }

    /// Read with the caller's buffer strategy (null = normal shared buffers),
    /// e.g. the ring VACUUM passes in `IndexVacuumInfo`.
    pub fn new(
        index: pg_sys::Relation,
        meta: &MetaPage,
        strategy: pg_sys::BufferAccessStrategy,
    ) -> Self {
        Self::open(index, meta, strategy, false)
    }

    /// `owned`: this source frees `strategy` when done.
    fn open(
        index: pg_sys::Relation,
        meta: &MetaPage,
        strategy: pg_sys::BufferAccessStrategy,
        owned: bool,
    ) -> Self {
        let mut blocks = Box::new(BlockRange {
            next: META_BLOCK + 1,
            last: META_BLOCK + meta.n_data_pages,
        });
        let stream = unsafe {
            pg_sys::read_stream_begin_relation(
                pg_sys::READ_STREAM_SEQUENTIAL as i32,
                strategy,
                index,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                Some(next_block),
                &mut *blocks as *mut BlockRange as *mut std::ffi::c_void,
                0,
            )
        };
        Self {
            stream,
            blocks,
            own_strategy: if owned { strategy } else { std::ptr::null_mut() },
        }
    }

    /// Release the stream (and our ring). Safe to call twice.
    fn finish(&mut self) {
        unsafe {
            if !self.stream.is_null() {
                pg_sys::read_stream_end(self.stream);
                self.stream = std::ptr::null_mut();
            }
            if !self.own_strategy.is_null() {
                pg_sys::FreeAccessStrategy(self.own_strategy);
                self.own_strategy = std::ptr::null_mut();
            }
        }
    }
}

/// On an ERROR, abort cleanup releases the pins and memory instead; calling
/// into Postgres while unwinding from one is not safe.
impl Drop for PageSource {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            self.finish();
        }
    }
}

impl ChunkSource for PageSource {
    fn next_chunk(&mut self) -> Option<Vec<u8>> {
        if self.stream.is_null() {
            return None;
        }
        unsafe {
            let buf = pg_sys::read_stream_next_buffer(self.stream, std::ptr::null_mut());
            if buf == pg_sys::InvalidBuffer as pg_sys::Buffer {
                debug_assert!(self.blocks.next > self.blocks.last);
                self.finish();
                return None;
            }
            pg_sys::LockBuffer(buf, pg_sys::BUFFER_LOCK_SHARE as i32);
            let src = contents(pg_sys::BufferGetPage(buf));
            let mut len = [0u8; 2];
            std::ptr::copy_nonoverlapping(src, len.as_mut_ptr(), 2);
            let len = u16::from_le_bytes(len) as usize;
            let chunk = std::slice::from_raw_parts(src.add(2), len.min(CHUNK_CAP)).to_vec();
            pg_sys::UnlockReleaseBuffer(buf);
            Some(chunk)
        }
    }
}

/// Allocate block 0 of a new index; it is filled in by `write_meta` later.
pub unsafe fn init_meta(index: pg_sys::Relation, fork: pg_sys::ForkNumber::Type) {
    unsafe {
        let buf = new_page(index, fork);
        assert_eq!(pg_sys::BufferGetBlockNumber(buf), META_BLOCK);
        write_meta_into(buf, &MetaPage::new(0, 0));
        pg_sys::MarkBufferDirty(buf);
        pg_sys::UnlockReleaseBuffer(buf);
    }
}

unsafe fn write_meta_into(buf: pg_sys::Buffer, meta: &MetaPage) {
    unsafe {
        let page = pg_sys::BufferGetPage(buf);
        std::ptr::write_unaligned(contents(page) as *mut MetaPage, *meta);
        let header = page as *mut pg_sys::PageHeaderData;
        (*header).pd_lower = (page_header_size() + std::mem::size_of::<MetaPage>()) as u16;
    }
}

pub unsafe fn write_meta(index: pg_sys::Relation, meta: &MetaPage) {
    unsafe {
        let buf = read_page(index, META_BLOCK, pg_sys::BUFFER_LOCK_EXCLUSIVE);
        write_meta_into(buf, meta);
        pg_sys::MarkBufferDirty(buf);
        pg_sys::UnlockReleaseBuffer(buf);
    }
}

pub unsafe fn read_meta(index: pg_sys::Relation) -> MetaPage {
    unsafe {
        let buf = read_page(index, META_BLOCK, pg_sys::BUFFER_LOCK_SHARE);
        let meta = std::ptr::read_unaligned(contents(pg_sys::BufferGetPage(buf)) as *const MetaPage);
        pg_sys::UnlockReleaseBuffer(buf);
        if meta.magic != META_MAGIC || meta.version != META_VERSION {
            pgrx::error!("tree_search_iam: index has a bad or outdated metapage; run REINDEX");
        }
        meta
    }
}

/// Flag the index as out of date. Cheap no-op when it already is.
pub unsafe fn mark_stale(index: pg_sys::Relation) {
    unsafe {
        let buf = read_page(index, META_BLOCK, pg_sys::BUFFER_LOCK_EXCLUSIVE);
        let page = pg_sys::BufferGetPage(buf);
        let mut meta = std::ptr::read_unaligned(contents(page) as *const MetaPage);
        if meta.stale == 0 {
            meta.stale = 1;
            write_meta_into(buf, &meta);
            pg_sys::MarkBufferDirty(buf);
        }
        pg_sys::UnlockReleaseBuffer(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    impl ChunkSink for Vec<Vec<u8>> {
        fn write_chunk(&mut self, chunk: &[u8]) {
            assert!(chunk.len() <= CHUNK_CAP);
            self.push(chunk.to_vec());
        }
    }

    impl ChunkSource for std::vec::IntoIter<Vec<u8>> {
        fn next_chunk(&mut self) -> Option<Vec<u8>> {
            self.next()
        }
    }

    fn uti(s: &str) -> UnifiedTreeIndex {
        UnifiedTreeIndex::parse(CString::new(s).unwrap().as_c_str()).unwrap()
    }

    /// A chain `{0{1{2...}}}` of `n` nodes, far bigger than one page when n is large.
    fn chain(n: usize) -> String {
        let mut s: String = (0..n).map(|i| format!("{{{i}")).collect();
        s.push_str(&"}".repeat(n));
        s
    }

    fn round_trip(trees: &[UnifiedTreeIndex]) -> (usize, Vec<(u64, UnifiedTreeIndex)>) {
        let mut w = StreamWriter::new(Vec::<Vec<u8>>::new());
        for (i, t) in trees.iter().enumerate() {
            w.push(i as u64 * 7 + 3, t);
        }
        let (chunks, n) = w.finish();
        assert_eq!(n as usize, trees.len());
        let n_chunks = chunks.len();
        let mut r = StreamReader::new(chunks.into_iter());
        let mut out = Vec::new();
        while let Some(e) = r.next_entry() {
            out.push(e);
        }
        (n_chunks, out)
    }

    #[test]
    fn empty_stream() {
        let (n_chunks, out) = round_trip(&[]);
        assert_eq!(n_chunks, 0);
        assert!(out.is_empty());
    }

    #[test]
    fn small_trees_round_trip() {
        let trees: Vec<_> = ["{a}", "{a{b}{c}}", "{r{a{b}{c}}{d{e}}}", "{x{y}{z}}"]
            .iter()
            .map(|s| uti(s))
            .collect();
        let (n_chunks, out) = round_trip(&trees);
        assert_eq!(n_chunks, 1);
        for (i, (tid, t)) in out.iter().enumerate() {
            assert_eq!(*tid, i as u64 * 7 + 3);
            assert_eq!(t, &trees[i]);
        }
    }

    /// Entries larger than a page and many entries crossing page boundaries.
    #[test]
    fn multi_page_round_trip() {
        let mut trees = vec![uti(&chain(3000))];
        for i in 0..2000 {
            trees.push(uti(&chain(1 + i % 40)));
        }
        trees.push(uti(&chain(5000)));
        let (n_chunks, out) = round_trip(&trees);
        assert!(n_chunks > 10, "expected many chunks, got {n_chunks}");
        assert_eq!(out.len(), trees.len());
        for (i, (tid, t)) in out.iter().enumerate() {
            assert_eq!(*tid, i as u64 * 7 + 3);
            assert_eq!(t, &trees[i]);
        }
    }

    #[test]
    fn raw_entries_decode_like_next_entry() {
        let trees: Vec<_> = (0..300).map(|i| uti(&chain(1 + (i * 37) % 900))).collect();
        let mut w = StreamWriter::new(Vec::<Vec<u8>>::new());
        for (i, t) in trees.iter().enumerate() {
            w.push(i as u64, t);
        }
        let (chunks, _) = w.finish();
        let mut r = StreamReader::new(chunks.into_iter());
        let mut raw = Vec::new();
        let mut i = 0;
        while r.next_raw_entry(&mut raw) {
            let (tid, t) = decode_entry(&raw);
            assert_eq!((tid, &t), (i as u64, &trees[i]));
            raw.clear();
            i += 1;
        }
        assert_eq!(i, trees.len());
    }

    #[test]
    fn bulkread_only_above_a_quarter_of_shared_buffers() {
        // 16384 buffers = 128 MB of 8 KB pages.
        assert!(!use_bulkread(0, 16384));
        assert!(!use_bulkread(4096, 16384));
        assert!(use_bulkread(4097, 16384));
        assert!(use_bulkread(u32::MAX, 16384));
    }

    #[test]
    fn tids_only() {
        let trees: Vec<_> = (0..500).map(|i| uti(&chain(1 + i % 700))).collect();
        let mut w = StreamWriter::new(Vec::<Vec<u8>>::new());
        for (i, t) in trees.iter().enumerate() {
            w.push(i as u64, t);
        }
        let (chunks, _) = w.finish();
        let mut r = StreamReader::new(chunks.into_iter());
        let tids: Vec<u64> = std::iter::from_fn(|| r.next_tid()).collect();
        assert_eq!(tids, (0..500).collect::<Vec<u64>>());
    }
}
