use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::common::list::SpecIter;

mod params;
mod spec;

pub use params::*;
pub use spec::*;

/// This struct describes a complete demos v2 file
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemosV2 {
    #[serde(with = "yaml_serde::with::singleton_map_recursive")]
    demos: IndexMap<String, DemoSpec>,
}

impl SpecIter<DemoSpec> for DemosV2 {
    fn inner(self) -> IndexMap<String, DemoSpec> {
        self.demos
    }
}

pub type List = crate::common::list::List<DemosV2, DemoSpec>;
