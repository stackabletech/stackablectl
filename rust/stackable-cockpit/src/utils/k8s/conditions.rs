use serde::Serialize;
use stackable_operator::{
    k8s_openapi::{
        api::apps::v1::{DeploymentCondition, StatefulSetCondition},
        apimachinery::pkg::apis::meta::v1::Condition,
    },
    status::condition::ClusterCondition,
};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct DisplayCondition {
    pub message: Option<String>,
    pub is_good: Option<bool>,
    pub condition: String,
}

impl DisplayCondition {
    pub fn new(condition: String, message: Option<String>, is_good: Option<bool>) -> Self {
        Self {
            condition,
            message,
            is_good,
        }
    }
}

/// This trait unifies the different conditions, like [`Condition`],
/// [`DeploymentCondition`], [`ClusterCondition`]. The method `plain` returns
/// a plain text representation of the list of conditions. This list is suited
/// for terminal output, i.e. stackablectl.
pub trait ConditionsExt
where
    Self: IntoIterator,
    Self::Item: ConditionExt,
{
    /// Returns a plain list of conditions.
    fn plain(&self) -> Vec<DisplayCondition>;
}

impl ConditionsExt for Vec<Condition> {
    fn plain(&self) -> Vec<DisplayCondition> {
        self.iter()
            .map(|c| {
                DisplayCondition::new(
                    format!(
                        "{condition_type}: {status}",
                        condition_type = c.type_,
                        status = c.status
                    ),
                    Some(c.message.clone()),
                    c.is_good(),
                )
            })
            .collect()
    }
}

impl ConditionsExt for Vec<DeploymentCondition> {
    fn plain(&self) -> Vec<DisplayCondition> {
        self.iter()
            .map(|c| {
                DisplayCondition::new(
                    format!(
                        "{condition_type}: {status}",
                        condition_type = c.type_,
                        status = c.status
                    ),
                    c.message.clone(),
                    c.is_good(),
                )
            })
            .collect()
    }
}

impl ConditionsExt for Vec<ClusterCondition> {
    fn plain(&self) -> Vec<DisplayCondition> {
        self.iter()
            .map(|c| DisplayCondition::new(c.display_short(), c.message.clone(), Some(c.is_good())))
            .collect()
    }
}

impl ConditionsExt for Vec<StatefulSetCondition> {
    fn plain(&self) -> Vec<DisplayCondition> {
        self.iter()
            .map(|c| {
                DisplayCondition::new(
                    format!(
                        "{condition_type}: {status}",
                        condition_type = c.type_,
                        status = c.status
                    ),
                    c.message.clone(),
                    c.is_good(),
                )
            })
            .collect()
    }
}

pub trait ConditionExt {
    fn is_good(&self) -> Option<bool> {
        None
    }
}

impl ConditionExt for StatefulSetCondition {}
impl ConditionExt for DeploymentCondition {}
impl ConditionExt for Condition {}

impl ConditionExt for ClusterCondition {
    fn is_good(&self) -> Option<bool> {
        Some(self.is_good())
    }
}
