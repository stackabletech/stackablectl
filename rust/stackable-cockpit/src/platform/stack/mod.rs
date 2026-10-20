use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

mod params;
mod spec;

pub use params::*;
pub use spec::*;

use crate::common::list::SpecIter;

/// This struct describes a complete demos v2 file
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StacksV2 {
    #[serde(with = "yaml_serde::with::singleton_map_recursive")]
    stacks: IndexMap<String, StackSpec>,
}

impl SpecIter<StackSpec> for StacksV2 {
    fn inner(self) -> IndexMap<String, StackSpec> {
        self.stacks
    }
}

pub type StackList = crate::common::list::List<StacksV2, StackSpec>;
