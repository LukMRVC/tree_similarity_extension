//! The `lb` reloption: which lower bound filters entries before TopDiff.

use std::ffi::{c_char, CStr};
use std::sync::atomic::{AtomicU32, Ordering};

use pgrx::pg_sys;
use pgrx::prelude::*;

use crate::pipelines::Lb;

static RELOPT_KIND: AtomicU32 = AtomicU32::new(0);

#[repr(C)]
struct TreeSearchIamOptions {
    vl_len_: i32,
    lb: i32,
}

const LB_NAME: &CStr = c"lb";

/// Register the `lb` option. Runs once per backend, from `_PG_init`.
pub fn register() {
    let names: [&'static CStr; 5] = [
        c"sed_struct",
        c"sed_plain",
        c"structural",
        c"binary_branch",
        c"lblint",
    ];
    // Postgres keeps this pointer, so the table lives for the whole backend.
    let members: &'static mut [pg_sys::relopt_enum_elt_def] = Box::leak(
        Lb::ALL
            .iter()
            .zip(names)
            .map(|(lb, name)| {
                debug_assert_eq!(lb.name().as_bytes(), name.to_bytes());
                pg_sys::relopt_enum_elt_def {
                    string_val: name.as_ptr(),
                    symbol_val: *lb as i32,
                }
            })
            .chain(std::iter::once(pg_sys::relopt_enum_elt_def {
                string_val: std::ptr::null(),
                symbol_val: 0,
            }))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    unsafe {
        let kind = pg_sys::add_reloption_kind();
        RELOPT_KIND.store(kind, Ordering::Relaxed);
        pg_sys::add_enum_reloption(
            kind,
            LB_NAME.as_ptr(),
            c"lower bound that filters entries before the exact TopDiff check".as_ptr(),
            members.as_mut_ptr(),
            Lb::SedStruct as i32,
            c"Valid values are \"sed_struct\", \"sed_plain\", \"structural\", \"binary_branch\" and \"lblint\".".as_ptr(),
            pg_sys::AccessExclusiveLock as pg_sys::LOCKMODE,
        );
    }
}

#[pg_guard]
pub unsafe extern "C-unwind" fn amoptions(reloptions: pg_sys::Datum, validate: bool) -> *mut pg_sys::bytea {
    let table = [pg_sys::relopt_parse_elt {
        optname: LB_NAME.as_ptr() as *const c_char,
        opttype: pg_sys::relopt_type::RELOPT_TYPE_ENUM,
        offset: std::mem::offset_of!(TreeSearchIamOptions, lb) as i32,
        ..Default::default()
    }];
    unsafe {
        pg_sys::build_reloptions(
            reloptions,
            validate,
            RELOPT_KIND.load(Ordering::Relaxed),
            std::mem::size_of::<TreeSearchIamOptions>(),
            table.as_ptr(),
            table.len() as i32,
        ) as *mut pg_sys::bytea
    }
}

/// The lower bound configured for this index (`sed_struct` when unset).
pub unsafe fn index_lb(index: pg_sys::Relation) -> Lb {
    unsafe {
        let opts = (*index).rd_options as *const TreeSearchIamOptions;
        if opts.is_null() {
            return Lb::SedStruct;
        }
        Lb::from_i32((*opts).lb).unwrap_or(Lb::SedStruct)
    }
}
