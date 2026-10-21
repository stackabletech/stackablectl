use yaml_serde::{Mapping, Value};

/// Extracts the Helm values for a specific operator from the operator values mapping.
///
/// Looks up the operator-specific key (e.g. `airflow-operator`) and returns
/// the associated mapping, or an empty mapping if not found.
pub fn values_for_operator(operator_values: &Mapping, operator_name: &str) -> Mapping {
    operator_values
        .get(format!("{operator_name}-operator"))
        .and_then(Value::as_mapping)
        .cloned()
        .unwrap_or_default()
}
