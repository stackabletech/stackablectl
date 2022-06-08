use crate::helpers::{c_str_ptr_to_str, GoString};
use crate::{CliArgs, NAMESPACE};
use cached::proc_macro::cached;
use lazy_static::lazy_static;
use log::{debug, error, info, warn, LevelFilter};
use serde::Deserialize;
use std::collections::HashMap;
use std::os::raw::c_char;
use std::process::exit;
use std::sync::Mutex;

lazy_static! {
    pub static ref HELM_REPOS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    pub static ref LOG_LEVEL: Mutex<LevelFilter> = Mutex::new(LevelFilter::Trace);
}

extern "C" {
    fn go_install_helm_release(
        release_name: GoString,
        chart_name: GoString,
        chart_version: GoString,
        namespace: GoString,
        supress_output: bool,
    );
    fn go_uninstall_helm_release(
        release_name: GoString,
        namespace: GoString,
        suppress_output: bool,
    );
    fn go_helm_release_exists(release_name: GoString, namespace: GoString) -> bool;
    fn go_helm_list_releases(namespace: GoString) -> *const c_char;
    fn go_add_helm_repo(name: GoString, url: GoString);
}

pub fn handle_common_cli_args(args: &CliArgs) {
    let mut repos = HELM_REPOS.lock().unwrap();
    repos.insert(
        "stackable-stable".to_string(),
        args.helm_repo_stackable_stable.clone(),
    );
    repos.insert(
        "stackable-test".to_string(),
        args.helm_repo_stackable_test.clone(),
    );
    repos.insert(
        "stackable-dev".to_string(),
        args.helm_repo_stackable_dev.clone(),
    );

    *(LOG_LEVEL.lock().unwrap()) = args.log_level;

    let namespace = &args.namespace;
    if namespace != "default" {
        info!("Deploying into non-default namespace.\
            Please make sure not to deploy the same operator multiple times in different namespaces unless you know what you are doing (TM).");
    }
    *(NAMESPACE.lock().unwrap()) = namespace.to_string();
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
        let helm_release = get_helm_release(release_name).unwrap_or_else(|| {
            panic!(
                "Failed to find helm release {release_name} besides helm saying it should be there"
            )
        });
        let current_version = helm_release.version;

        match chart_version {
            None => {
                warn!("The release {release_name} in version {current_version} is already installed and you have not requested a specific version, not re-installing it");
                return;
            }
            Some(chart_version) => {
                if chart_version == current_version {
                    info!("The release {release_name} in version {current_version} is already installed, not installing it");
                    return;
                } else {
                    error!("The helm release {release_name} is already installed in version {current_version} but you requested to install it in version {chart_version}. \
                    Use \"stackablectl operator uninstall {operator_name}\" to uninstall it.");
                    exit(1);
                }
            }
        }
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

/// Cached because of slow network calls
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

pub fn uninstall_helm_release(release_name: &str) {
    if helm_release_exists(release_name) {
        unsafe {
            go_uninstall_helm_release(
                GoString::from(release_name),
                GoString::from(NAMESPACE.lock().unwrap().as_str()),
                *LOG_LEVEL.lock().unwrap() < LevelFilter::Debug,
            );
        }
    } else {
        warn!("The helm release {release_name} is not installed, not removing.");
    }
}

fn install_helm_release(release_name: &str, chart_name: &str, chart_version: &str) {
    unsafe {
        go_install_helm_release(
            GoString::from(release_name),
            GoString::from(chart_name),
            GoString::from(chart_version),
            GoString::from(NAMESPACE.lock().unwrap().as_str()),
            *LOG_LEVEL.lock().unwrap() < LevelFilter::Debug,
        )
    }
}

fn helm_release_exists(release_name: &str) -> bool {
    unsafe {
        go_helm_release_exists(
            GoString::from(release_name),
            GoString::from(NAMESPACE.lock().unwrap().as_str()),
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub name: String,
    pub version: String,
    pub namespace: String,
    pub status: String,
    pub last_updated: String,
}

pub fn helm_list_releases() -> Vec<Release> {
    let releases_json = c_str_ptr_to_str(unsafe {
        go_helm_list_releases(GoString::from(NAMESPACE.lock().unwrap().as_str()))
    });

    serde_json::from_str(releases_json).unwrap_or_else(|_| {
        panic!("Failed to parse helm releases JSON from go wrapper {releases_json}")
    })
}

pub fn get_helm_release(release_name: &str) -> Option<Release> {
    helm_list_releases()
        .into_iter()
        .find(|release| release.name == release_name)
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
