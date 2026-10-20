use yaml_serde::Mapping;
use stackable_operator::kvp::Labels;

use crate::platform::operator::ChartSourceType;

#[derive(Debug)]
pub struct StackInstallParameters {
    /// Name of the stack, which is always present
    pub stack_name: String,

    /// Optional name of the demo, which is only present in case this stack is installed as part of
    /// a demo. This is unset in case a stack is installed directly.
    pub demo_name: Option<String>,

    pub operator_namespace: String,
    pub stack_namespace: String,

    pub parameters: Vec<String>,
    pub skip_release: bool,
    pub labels: Labels,
    pub chart_source: ChartSourceType,
    pub operator_values: Mapping,
}

pub struct StackUninstallParameters {
    pub stack_name: String,

    /// Optional name of the demo, which is only present in case this stack is uninstalled as part of
    /// a demo. This is unset in case a stack is uninstalled directly.
    pub demo_name: Option<String>,

    pub operator_namespace: String,
    pub stack_namespace: String,

    pub skip_operators: bool,
    pub skip_crds: bool,
}
