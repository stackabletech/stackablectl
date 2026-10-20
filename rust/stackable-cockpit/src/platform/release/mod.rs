use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

mod spec;

pub use spec::*;

use crate::common::list::SpecIter;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Releases {
    #[serde(with = "yaml_serde::with::singleton_map_recursive")]
    releases: IndexMap<String, ReleaseSpec>,
}

impl SpecIter<ReleaseSpec> for Releases {
    fn inner(self) -> IndexMap<String, ReleaseSpec> {
        self.releases
    }
}

pub type ReleaseList = crate::common::list::List<Releases, ReleaseSpec>;
