use pgrx::itemptr::item_pointer_to_u64;
use pgrx::prelude::*;
use pgrx::{pg_sys, PgMemoryContexts};

use super::storage::{self, MetaPage, PageSink, StreamWriter};
use crate::types::Tree;

struct BuildState {
    writer: StreamWriter<PageSink>,
    heap_tuples: f64,
    tmp_ctx: PgMemoryContexts,
}

#[pg_guard]
pub unsafe extern "C-unwind" fn ambuild(
    heap: pg_sys::Relation,
    index: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe {
        if pg_sys::RelationGetNumberOfBlocksInFork(index, pg_sys::ForkNumber::MAIN_FORKNUM) != 0 {
            error!("index \"{}\" already contains data", rel_name(index));
        }
        storage::init_meta(index, pg_sys::ForkNumber::MAIN_FORKNUM);

        let mut state = BuildState {
            writer: StreamWriter::new(PageSink::new(index)),
            heap_tuples: 0.0,
            tmp_ctx: PgMemoryContexts::new("tree_search_iam build"),
        };
        pg_sys::IndexBuildHeapScan(heap, index, index_info, Some(build_callback), &mut state);

        let (sink, n_entries) = state.writer.finish();
        storage::write_meta(index, &MetaPage::new(n_entries, sink.n_pages));

        let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
        result.heap_tuples = state.heap_tuples;
        result.index_tuples = n_entries as f64;
        result.into_pg()
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn build_callback(
    _index: pg_sys::Relation,
    tid: pg_sys::ItemPointer,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    _tuple_is_alive: bool,
    state: *mut std::ffi::c_void,
) {
    unsafe {
        let state = &mut *(state as *mut BuildState);
        state.heap_tuples += 1.0;
        let (datum, is_null) = (*values, *isnull);
        // NULL trees can never match `<~`, so they are left out.
        if is_null {
            return;
        }
        let tree = state.tmp_ctx.switch_to(|_| Tree::from_datum(datum, false));
        if let Some(tree) = tree {
            state.writer.push(item_pointer_to_u64(*tid), &tree.to_unified());
        }
        state.tmp_ctx.reset();
    }
}

#[pg_guard]
pub unsafe extern "C-unwind" fn ambuildempty(_index: pg_sys::Relation) {
    error!("tree_search_iam does not support unlogged tables");
}

pub(super) unsafe fn rel_name(rel: pg_sys::Relation) -> String {
    unsafe {
        std::ffi::CStr::from_ptr((*(*rel).rd_rel).relname.data.as_ptr())
            .to_string_lossy()
            .into_owned()
    }
}
