//! Two-stage LB-filter → TopDiff-verification pipelines over `UnifiedTreeIndex`,
//! one per Stage-1 lower-bound filter. All share one contract: the exact TED
//! when it is `<= k`, otherwise `k + 1`.

pub mod binary_branch;
pub mod lblint;
pub mod sed_plain;
pub mod sed_struct;
pub mod structural;

use crate::types::UnifiedTreeIndex;

/// Which Stage-1 lower bound runs before the exact TopDiff check. The result is
/// the same for every variant; only the speed differs.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lb {
    SedStruct = 0,
    SedPlain = 1,
    Structural = 2,
    BinaryBranch = 3,
    Lblint = 4,
}

impl Lb {
    pub const ALL: [Lb; 5] = [
        Lb::SedStruct,
        Lb::SedPlain,
        Lb::Structural,
        Lb::BinaryBranch,
        Lb::Lblint,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Lb::SedStruct => "sed_struct",
            Lb::SedPlain => "sed_plain",
            Lb::Structural => "structural",
            Lb::BinaryBranch => "binary_branch",
            Lb::Lblint => "lblint",
        }
    }

    pub fn from_i32(v: i32) -> Option<Lb> {
        Lb::ALL.into_iter().find(|lb| *lb as i32 == v)
    }

    /// TED(query, cand) when it is `<= k`, otherwise `k + 1`.
    pub fn within(self, query: &UnifiedTreeIndex, cand: &UnifiedTreeIndex, k: i32) -> i32 {
        match self {
            Lb::SedStruct => sed_struct::sed_struct_within(query, cand, k),
            Lb::SedPlain => sed_plain::sed_plain_within(query, cand, k),
            Lb::Structural => structural::structural_within(query, cand, k),
            Lb::BinaryBranch => binary_branch::binary_branch_within(query, cand, k),
            Lb::Lblint => lblint::lblint_within(query, cand, k),
        }
    }
}
