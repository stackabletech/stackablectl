use crate::helpers::GoString;
use log::{debug, error, warn};
use phf::phf_map;
use std::process::exit;

/// key: Helm repo name
/// value: Helm repo url
const HELM_REPOS: phf::Map<&'static str, &'static str> = phf_map! {
    "stackable-dev" => "https://repo.stackable.tech/repository/helm-dev",
    "stackable-test" => "https://repo.stackable.tech/repository/helm-test",
    "stackable-stable" => "https://repo.stackable.tech/repository/helm-stable",
    "prometheus-community" =>"https://prometheus-community.github.io/helm-charts",
};

extern "C" {
    fn go_install_helm_release(
        release_name: GoString,
        chart_name: GoString,
        chart_version: GoString,
    );
    fn go_helm_release_exists(release_name: GoString) -> bool;
    fn go_add_helm_repo(name: GoString, url: GoString);
}

// Wrapper around the go functions
fn install_helm_release(release_name: &str, chart_name: &str, chart_version: &str) {
    unsafe {
        go_install_helm_release(
            GoString::from(release_name),
            GoString::from(chart_name),
            GoString::from(chart_version),
        )
    }
}

fn helm_release_exists(release_name: &str) -> bool {
    unsafe { go_helm_release_exists(GoString::from(release_name)) }
}

fn add_helm_repo(name: &str, url: &str) {
    unsafe { go_add_helm_repo(GoString::from(name), GoString::from(url)) }
}

/// Installs the specified helm chart with the release_name
/// If the release is already running it errors out (maybe in the future prompt the user if it should be deleted)
/// If the chart_version is None the version `>0.0.0-0` will be used.
/// This is equivalent to the `helm install` flag `--devel`.
pub fn install_helm_release_from_repo(
    release_name: &str,
    repo_name: &str,
    chart_name: &str,
    chart_version: Option<&str>,
) {
    if helm_release_exists(release_name) {
        error!("The helm release {release_name} is already running. Use \"helm uninstall {release_name}\" to uninstall it.");
        warn!("TODO: Implement user prompt to delete it for him");
        exit(1);
    }

    match HELM_REPOS.get_entry(repo_name) {
        None => {
            error!("I don't know about the helm repo {repo_name}");
            exit(1);
        }
        Some((repo_name, repo_url)) => {
            debug!("Adding helm repo {repo_name} with URL {repo_url}");
            add_helm_repo(repo_name, repo_url);
        }
    }

    let full_chart_name = format!("{repo_name}/{chart_name}");
    let chart_version = chart_version.unwrap_or(">0.0.0-0");
    debug!("Installing helm release {repo_name} from chart {full_chart_name} in version {chart_version}");
    install_helm_release(release_name, &full_chart_name, chart_version);
}
