use snafu::Snafu;
use stackable_operator::kube;

use crate::utils::k8s::{self, Client};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to create Kubernetes client"))]
    KubeClientCreate { source: Box<k8s::Error> },

    #[snafu(display(
        "permission denied - try to create the namespace manually or choose an already existing one to which you have access to"
    ))]
    PermissionDenied,
}

/// Creates a namespace with `name` if needed (not already present in the
/// cluster).
// TODO (@NickLarsenNZ): Take a &str instead of String (to avoid all the cloning)
pub async fn create_if_needed(client: &Client, name: String) -> Result<(), Error> {
    client
        .create_namespace_if_needed(name)
        .await
        .map_err(|err| match err {
            k8s::Error::KubeClientCreate { source } => match &*source {
                kube::Error::Api(err) if err.code == 401 => Error::PermissionDenied,
                _ => Error::KubeClientCreate {
                    source: Box::new(k8s::Error::KubeClientCreate { source }),
                },
            },
            _ => Error::KubeClientCreate {
                source: Box::new(err),
            },
        })
}
