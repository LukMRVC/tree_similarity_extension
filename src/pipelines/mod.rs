//! Two-stage LB-filter → TopDiff-verification pipelines over `UnifiedTreeIndex`,
//! one per Stage-1 lower-bound filter. The SED-Struct variant lives in lib.rs
//! (`sed_topdiff_within`); these mirror its contract with different filters.

pub mod binary_branch;
pub mod lblint;
pub mod sed_plain;
pub mod structural;
