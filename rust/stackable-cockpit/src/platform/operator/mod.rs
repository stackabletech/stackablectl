use std::{fmt::Display, str::FromStr};

use listener_operator::LISTENER_CLASS_PRESET;
use semver::Version;
use serde::Serialize;
use yaml_serde::{Mapping, Value};
use snafu::{ResultExt, Snafu, ensure};
use tracing::{Span, info, instrument};
use tracing_indicatif::{indicatif_println, span_ext::IndicatifSpanExt};

use crate::{
    constants::{
        HELM_OCI_REGISTRY, HELM_REPO_NAME_DEV, HELM_REPO_NAME_STABLE, HELM_REPO_NAME_TEST,
    },
    helm,
    utils::operator_chart_name,
};

pub mod listener_operator;

pub const VALID_OPERATORS: &[&str] = &[
    "airflow",
    "commons",
    "druid",
    "hbase",
    "hdfs",
    "hello-world",
    "hive",
    "kafka",
    "listener",
    "nifi",
    "opa",
    "opensearch",
    "secret",
    "spark-k8s",
    "superset",
    "trino",
    "zookeeper",
];

#[derive(Debug, Snafu)]
pub enum SpecParseError {
    #[snafu(display("invalid equal sign count in operator spec, expected one"))]
    InvalidEqualSignCount,

    #[snafu(display("failed to parse SemVer version"))]
    ParseVersion { source: semver::Error },

    #[snafu(display("the operator spec includes '=' but no version was specified"))]
    MissingVersion,

    #[snafu(display("empty operator spec input"))]
    EmptyInput,

    #[snafu(display(
        "invalid operator name {name:?}. \
        It could be the case that this version of stackablectl is too old to know about this particular operator, \
        in which case you should update it."
    ))]
    InvalidName { name: String },
}

/// OperatorSpec describes the format of an operator name with optional version
/// number. The string format is `<OPERATOR_NAME>(=<VERSION>)`. Valid values
/// are: `operator`, `operator=1.2.3` or `operator=1.2.3-rc1`.
#[derive(Clone, Debug)]
pub struct OperatorSpec {
    pub version: Option<Version>,
    pub name: String,
}

impl Display for OperatorSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{name}{version_selector}",
            name = self.name,
            version_selector = match &self.version {
                Some(v) => format!("={v}"),
                None => "".into(),
            }
        )
    }
}

impl FromStr for OperatorSpec {
    type Err = SpecParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.trim();

        // Empty input is not allowed
        ensure!(!input.is_empty(), EmptyInputSnafu);

        // Split at each equal sign
        let parts: Vec<&str> = input.split('=').collect();
        let len = parts.len();

        // If there are more than 2 equal signs, return error
        // because of invalid spec format
        ensure!(len <= 2, InvalidEqualSignCountSnafu);

        // Check if the provided operator name is in the list of valid operators
        ensure!(
            VALID_OPERATORS.contains(&parts[0]),
            InvalidNameSnafu { name: parts[0] }
        );

        // If there is only one part, the input didn't include
        // the optional version identifier
        if len == 1 {
            return Ok(Self {
                name: input.into(),
                version: None,
            });
        }

        // If there is an equal sign, but no version after
        ensure!(!parts[1].is_empty(), MissingVersionSnafu);

        // There are two parts, so an operator name and version
        let version: Version = parts[1].parse().context(ParseVersionSnafu)?;

        Ok(Self {
            name: parts[0].into(),
            version: Some(version),
        })
    }
}

impl TryFrom<String> for OperatorSpec {
    type Error = SpecParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl TryFrom<&str> for OperatorSpec {
    type Error = SpecParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl OperatorSpec {
    pub fn new<T>(name: T, version: Option<Version>) -> Result<Self, SpecParseError>
    where
        T: AsRef<str>,
    {
        let name = name.as_ref();

        if !VALID_OPERATORS.contains(&name) {
            return Err(SpecParseError::InvalidName {
                name: name.to_string(),
            });
        }

        Ok(Self {
            name: name.to_string(),
            version,
        })
    }

    /// Returns the name used by Helm
    pub fn helm_name(&self) -> String {
        operator_chart_name(&self.name)
    }

    /// Returns the repo used by Helm based on the specified version
    pub fn helm_repo_name(&self) -> String {
        match &self.version {
            Some(version) => match version.pre.as_str() {
                "nightly" => HELM_REPO_NAME_DEV,
                "dev" => HELM_REPO_NAME_DEV,
                v => {
                    if v.starts_with("pr") {
                        HELM_REPO_NAME_TEST
                    } else {
                        HELM_REPO_NAME_STABLE
                    }
                }
            },
            None => HELM_REPO_NAME_DEV,
        }
        .into()
    }

    /// Installs the operator using Helm.
    #[instrument(skip_all, fields(
        %namespace,
        name = %self.name,
        // NOTE (@NickLarsenNZ): Option doesn't impl Display, so we need to call
        // display for the inner type if it exists. Otherwise we gte the Debug
        // impl for the whole Option.
        version = self.version.as_ref().map(tracing::field::display),
        indicatif.pb_show = true
    ))]
    pub fn install(
        &self,
        namespace: &str,
        chart_source: &ChartSourceType,
        values: &Mapping,
    ) -> Result<(), helm::Error> {
        info!(operator = %self, "Installing operator");
        Span::current()
            .pb_set_message(format!("Installing {name}-operator", name = self.name).as_str());

        let version = self.version.as_ref().map(|v| v.to_string());
        let helm_name = self.helm_name();

        // we can't resolve this any earlier as, for the repository case,
        // this will be dependent on the operator version.
        let chart_source = match chart_source {
            ChartSourceType::OCI => HELM_OCI_REGISTRY.to_string(),
            ChartSourceType::Repo => self.helm_repo_name(),
        };

        let mut helm_values = values.clone();
        if self.name == "listener" {
            let preset = LISTENER_CLASS_PRESET
                .get()
                .expect("LISTENER_CLASS_PRESET must have been set")
                .as_helm_value();
            helm_values.insert(Value::String("preset".to_string()), preset);
        }

        let helm_values_yaml = if helm_values.is_empty() {
            None
        } else {
            Some(
                yaml_serde::to_string(&helm_values)
                    .expect("serializing a small YAML Mapping back to a YAML string can't fail"),
            )
        };

        // Install using Helm
        helm::install_release_from_repo_or_registry(
            &helm_name,
            helm::ChartVersion {
                chart_version: version.as_deref(),
                chart_name: &helm_name,
                chart_source: &chart_source,
            },
            helm_values_yaml.as_deref(),
            namespace,
            true,
        )?;

        Ok(())
    }

    /// Uninstalls the operator using Helm.
    #[instrument(skip_all, fields(%namespace))]
    pub fn uninstall<T>(&self, namespace: T) -> Result<(), helm::Error>
    where
        T: AsRef<str> + std::fmt::Display + std::fmt::Debug,
    {
        match helm::uninstall_release(&self.helm_name(), namespace.as_ref(), true) {
            Ok(status) => {
                indicatif_println!("{status}");
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChartSourceType {
    /// OCI registry
    OCI,

    /// index.yaml-based repositories: resolution (dev, test, stable) is based on the version and thus may be operator-specific
    Repo,
}

#[cfg(test)]
mod test {
    use rstest::rstest;
    use semver::Version;

    use crate::{
        constants::{HELM_REPO_NAME_DEV, HELM_REPO_NAME_STABLE, HELM_REPO_NAME_TEST},
        platform::operator::{OperatorSpec, SpecParseError},
    };

    #[test]
    fn simple_operator_spec() {
        match OperatorSpec::try_from("airflow") {
            Ok(spec) => {
                assert_eq!(spec.name, String::from("airflow"));
                assert_eq!(spec.version, None);
            }
            Err(err) => panic!("{err}"),
        }
    }

    #[test]
    fn version_operator_spec() {
        match OperatorSpec::try_from("zookeeper=1.2.3") {
            Ok(spec) => {
                assert_eq!(spec.name, String::from("zookeeper"));
                assert_eq!(spec.version, Some(Version::new(1, 2, 3)));
            }
            Err(err) => panic!("{err}"),
        }
    }

    #[test]
    fn empty_operator_spec() {
        match OperatorSpec::try_from("") {
            Ok(spec) => panic!("SHOULD FAIL: {spec}"),
            Err(err) => assert!(matches!(err, SpecParseError::EmptyInput)),
        }
    }

    #[test]
    fn empty_version_operator_spec() {
        match OperatorSpec::try_from("airflow=") {
            Ok(spec) => panic!("SHOULD FAIL: {spec}"),
            Err(err) => assert!(matches!(err, SpecParseError::MissingVersion)),
        }
    }

    #[test]
    fn invalid_version_operator_spec() {
        match OperatorSpec::try_from("airflow=1.2.3=") {
            Ok(spec) => panic!("SHOULD FAIL: {spec}"),
            Err(err) => assert!(matches!(err, SpecParseError::InvalidEqualSignCount)),
        }
    }

    #[rstest]
    #[case("airflow=0.0.0-nightly", HELM_REPO_NAME_DEV)]
    #[case("airflow=0.0.0-pr123", HELM_REPO_NAME_TEST)]
    #[case("airflow=0.0.0-dev", HELM_REPO_NAME_DEV)]
    #[case("airflow=1.2.3", HELM_REPO_NAME_STABLE)]
    #[case("airflow", HELM_REPO_NAME_DEV)]
    fn repo_name(#[case] input: &str, #[case] repo: &str) {
        let spec = OperatorSpec::try_from(input).unwrap();
        assert_eq!(spec.helm_repo_name(), repo);
    }
}
