use crate::helpers;
use log::{debug, error};

pub const HELM_DEV_REPO_URL: &str = "https://repo.stackable.tech/repository/helm-dev";
pub const HELM_TEST_REPO_URL: &str = "https://repo.stackable.tech/repository/helm-test";
pub const HELM_STABLE_REPO_URL: &str = "https://repo.stackable.tech/repository/helm-stable";

pub fn install_helm_release<'a>(
    name: &'a str,
    repo_url: &'a str,
    mut additional_helm_args: Vec<&'a str>,
) {
    if check_if_helm_release_exists(name) {
        error!("The operator {name} is already running in the helm release {name}. Use \"helm uninstall {name}\" to uninstall it.");
        panic!();
    } else {
        // TODO Check if directory with same name exists. If it does print at least a WARN
        debug!("Installing release {name}");
        let mut args = vec!["helm", "install", "--repo", repo_url, name, name];
        args.append(&mut additional_helm_args);
        helpers::execute_command(args);
    }
}

fn check_if_helm_release_exists(name: &str) -> bool {
    helpers::execute_command_and_return_exit_code(vec!["helm", "status", name]) == 0
}

// pub fn add_and_update_repo(name: &str, url: &str) {
//     debug!("Adding Helm repo {name} with URL {url}");
//     helpers::execute_command(vec!["helm", "repo", "add", name, url]);
//     debug!("Updating Helm repo {name}");
//     helpers::execute_command(vec!["helm", "repo", "update", name]);
// }

// Adds Helm repository if it does not exist yet (it looks for a repository with the same URL, not name).
// An `update` command will also be run in either case.
// Returns the name of the repository, might differ from the passed name if it did already exist
// pub fn add_repo(name: &str, url: &str) -> String {
//     debug!("Checking whether Helm repository {name} already exists");
//
//     #[derive(Serialize, Deserialize, Debug)]
//     struct Repo {
//         name: String,
//         url: String,
//     }
//
//     let repos_json = helpers::execute_command(vec!["helm", "repo", "list", "-o", "json"]);
//     let repos: Vec<Repo> = serde_json::from_str(&repos_json).expect("Could not parse helm repo json");
//     trace!("Found the repos {:?}", repos);
//
//     let repo = repos.iter().filter(|repo| repo.name == name).next();
//
//     let repo_name = match repo {
//         None => {
//             debug!("Helm repository {name} (URL {url}) missing - adding now");
//             "TODO".to_string()
//         }
//         Some(repo) => {
//             debug!("Found existing repository {} with URL {}", repo.name, repo.url);
//
//             repo.name.to_string()
//         }
//     };
//
//     debug!("Updating repository {}", &repo_name);
//     helpers::execute_command(vec!["helm", "repo", "update", &repo_name]);
//
//     repo_name
// }
