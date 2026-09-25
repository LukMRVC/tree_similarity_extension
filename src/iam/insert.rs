//! Writes after the build are not indexed: they only mark the index stale.

use pgrx::itemptr::u64_to_item_pointer;
use pgrx::pg_sys;
use pgrx::prelude::*;

use super::storage::{self, PageSource, StreamReader};

#[pg_guard]
pub unsafe extern "C-unwind" fn aminsert(
    index: pg_sys::Relation,
    _values: *mut pg_sys::Datum,
    _isnull: *mut bool,
    _heap_tid: pg_sys::ItemPointer,
    _heap: pg_sys::Relation,
    _check_unique: pg_sys::IndexUniqueCheck::Type,
    _index_unchanged: bool,
    _index_info: *mut pg_sys::IndexInfo,
) -> bool {
    unsafe { storage::mark_stale(index) };
    false
}

/// VACUUM: the index can't drop entries, so any dead TID makes it stale.
#[pg_guard]
pub unsafe extern "C-unwind" fn ambulkdelete(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    callback: pg_sys::IndexBulkDeleteCallback,
    callback_state: *mut std::ffi::c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        let index = (*info).index;
        let stats = if stats.is_null() {
            PgBox::<pg_sys::IndexBulkDeleteResult>::alloc0().into_pg()
        } else {
            stats
        };
        let meta = storage::read_meta(index);
        let callback = callback.expect("ambulkdelete without a callback");
        let mut reader = StreamReader::new(PageSource::new(index, &meta));
        let mut any_dead = false;
        let mut tid = pg_sys::ItemPointerData::default();
        while let Some(raw) = reader.next_tid() {
            u64_to_item_pointer(raw, &mut tid);
            if callback(&mut tid, callback_state) {
                any_dead = true;
                (*stats).tuples_removed += 1.0;
            }
        }
        if any_dead {
            storage::mark_stale(index);
        }
        (*stats).num_index_tuples = meta.n_entries as f64;
        (*stats).num_pages = meta.n_data_pages + 1;
        stats
    }
}

#[pg_guard]
pub unsafe extern "C-unwind" fn amvacuumcleanup(
    _info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    stats
}
