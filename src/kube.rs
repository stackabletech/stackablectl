// /// This function currently uses `kubectl apply`.
// /// In the future we want to switch to kube-rs or something else to not require the user to install kubectl.
// pub fn deploy_manifest(yaml: &str) {
//     helpers::execute_command_with_stdin(vec!["kubectl", "apply", "-f", "-"], yaml);
// }

// use crate::kube::Error::TypelessManifest;
// use kube::api::{DynamicObject, GroupVersionKind, TypeMeta};
// use kube::{Client, Discovery};
// use snafu::{OptionExt, ResultExt, Snafu};
//
// pub const TEST: &str = r#"
// apiVersion: monitoring.coreos.com/v1
// kind: ServiceMonitor
// foo:
// metadata:
//   name: scrape-label
//   labels:
//     release: prometheus-operator
// spec:
//   endpoints:
//   - port: metrics
//   jobLabel: app.kubernetes.io/instance
//   selector:
//     matchLabels:
//       prometheus.io/scrape: "true"
//   namespaceSelector:
//     any: true
// "#;
//
// #[derive(Snafu, Debug)]
// pub enum Error {
//     #[snafu(display("failed to create kubernetes client"))]
//     CreateClient { source: kube::Error },
//     #[snafu(display("failed to parse manifest {manifest}"))]
//     ParseManifest {
//         source: serde_yaml::Error,
//         manifest: String,
//     },
//     #[snafu(display("manifest {manifest} has no type"))]
//     TypelessManifest { manifest: String },
// }
//
// // see https://gitlab.com/teozkr/thruster/-/blob/35b6291788fa209c52dd47fe6c96e1b483071793/src/apply.rs#L121-145
// pub async fn deploy_manifest(yaml: &str) -> Result<(), Error> {
//     let manifest = serde_yaml::from_str::<DynamicObject>(yaml).context(ParseManifestSnafu {
//         manifest: yaml.to_string(),
//     })?;
//     let manifest_type = manifest.types.as_ref().context(TypelessManifestSnafu {manifest: yaml})?;
//     let gvk = gvk_of_typemeta(manifest_type);
//
//     let client = create_client().await?;
//
//     Ok(())
// }
//
// async fn create_client() -> Result<Client, Error> {
//     Client::try_default().await.context(CreateClientSnafu)
// }
//
// fn gvk_of_typemeta(tpe: &TypeMeta) -> GroupVersionKind {
//     match tpe.api_version.split_once('/') {
//         Some((group, version)) => GroupVersionKind::gvk(&group, &version, &tpe.kind),
//         None => GroupVersionKind::gvk("", &tpe.api_version, &tpe.kind),
//     }
// }
