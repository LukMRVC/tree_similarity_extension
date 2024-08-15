use pgrx::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PostgresType, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Node;
