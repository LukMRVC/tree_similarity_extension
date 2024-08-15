use super::node::Node;
use pgrx::prelude::*;
use serde::*;

#[derive(PostgresType, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[inoutfuncs]
pub struct TreeArena {
    nodes: Vec<Node>,
}

impl InOutFuncs for TreeArena {
    fn input(input: &core::ffi::CStr) -> Self
    where
        Self: Sized,
    {
        todo!("Implement input parsing for TreeArena.")
    }

    fn output(&self, buffer: &mut pgrx::StringInfo) {
        todo!("Implement output dipslay for TreeArena")
    }
}
