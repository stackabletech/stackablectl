use crate::helpers::GoString;
use cached::proc_macro::cached;
use lazy_static::lazy_static;
use log::{debug, error, warn};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::exit;
use crate::CliArgs;
use std::sync::Mutex;

lazy_static! {
    pub static ref HELM_REPOS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

extern "C" {
    fn go_install_helm_release(
        release_name: GoString,
        chart_name: GoString,
        chart_version: GoString,
    );
    fn go_uninstall_helm_release(release_name: GoString);
    fn go_helm_release_exists(release_name: GoString) -> bool;
    fn go_add_helm_repo(name: GoString, url: GoString);
}

pub fn add_helm_repos_from_cli_args(args: &CliArgs) {
    let mut repos = HELM_REPOS.lock().unwrap();
    repos.insert("stackable-stable".to_string(), args.helm_repo_stackable_stable.clone());
    repos.insert("stackable-test".to_string(), args.helm_repo_stackable_test.clone());
    repos.insert("stackable-dev".to_string(), args.helm_repo_stackable_dev.clone());
}

/// Installs the specified helm chart with the release_name
/// If the release is already running it errors out (maybe in the future prompt the user if it should be deleted)
/// If the chart_version is None the version `>0.0.0-0` will be used.
/// This is equivalent to the `helm install` flag `--devel`.
pub fn install_helm_release_from_repo(
    operator_name: &str,
    release_name: &str,
    repo_name: &str,
    chart_name: &str,
    chart_version: Option<&str>,
) {
    if helm_release_exists(release_name) {
        error!("The helm release {release_name} is already running. Use \"stackablectl operator uninstall {operator_name}\" to uninstall it.");
        warn!("TODO: Implement user prompt to delete it for him");
        exit(1);
    }

    match HELM_REPOS.lock().unwrap().get(repo_name) {
        None => {
            error!("I don't know about the helm repo {repo_name}");
            exit(1);
        }
        Some(repo_url) => {
            debug!("Adding helm repo {repo_name} with URL {repo_url}");
            add_helm_repo(repo_name, repo_url);
        }
    }

    let full_chart_name = format!("{repo_name}/{chart_name}");
    let chart_version = chart_version.unwrap_or(">0.0.0-0");
    debug!("Installing helm release {repo_name} from chart {full_chart_name} in version {chart_version}");
    install_helm_release(release_name, &full_chart_name, chart_version);
}

pub fn uninstall_helm_release(release_name: &str) {
    if helm_release_exists(release_name) {
        unsafe {
            go_uninstall_helm_release(GoString::from(release_name));
        }
    } else {
        warn!("The helm release {release_name} is not installed, not removing.");
    }
}

#[cached]
pub fn get_repo_index(repo_url: String) -> HelmRepo {
    let index_url = format!("{repo_url}/index.yaml");
    debug!("Fetching helm repo index from {index_url}");

    let resp = reqwest::blocking::get(&index_url)
        .unwrap_or_else(|_| panic!("Failed to download helm repo index from {index_url}"))
        .text()
        .unwrap_or_else(|_| panic!("Failed to get text from {index_url}"));

    serde_yaml::from_str(&resp)
        .unwrap_or_else(|_| panic!("Failed to parse helm repo index from {index_url}"))
}

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

#[derive(Clone, Debug, Deserialize)]
pub struct HelmRepo {
    pub entries: HashMap<String, Vec<HelmRepoEntry>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HelmRepoEntry {
    pub name: String,
    pub version: String,
}
