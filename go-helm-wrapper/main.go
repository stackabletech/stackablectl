package main

import (
    "C"
    "context"
    "encoding/json"
    "time"
    "helm.sh/helm/v3/pkg/action"
    "helm.sh/helm/v3/pkg/repo"
    gohelm "github.com/mittwald/go-helm-client"

    // Needed for authentication against clusters, e.g. GCP
    // see https://github.com/kubernetes/client-go/issues/242
    _ "k8s.io/client-go/plugin/pkg/client/auth"
)

func main() {

}

//export go_install_helm_release
func go_install_helm_release(releaseName string, chartName string, chartVersion string, namespace string, suppressOutput bool) {
    helmClient := getHelmClient(namespace, suppressOutput)

    timeout, _ := time.ParseDuration("10m")
    chartSpec := gohelm.ChartSpec{
        ReleaseName: releaseName,
        ChartName:   chartName,
        Version:     chartVersion,
        Namespace:   namespace,
        UpgradeCRDs: true,
        Wait:        true,
        Timeout:     timeout,
    }

    if _, err := helmClient.InstallChart(context.Background(), &chartSpec, nil); err != nil {
        panic(err)
    }
}

//export go_uninstall_helm_release
func go_uninstall_helm_release(releaseName string, namespace string, suppressOutput bool) {
    helmClient := getHelmClient(namespace, suppressOutput)

    if err := helmClient.UninstallReleaseByName(releaseName); err != nil {
        panic(err)
    }
}

//export go_helm_release_exists
func go_helm_release_exists(releaseName string, namespace string) bool {
    helmClient := getHelmClient(namespace, true)

    release, _ := helmClient.GetRelease(releaseName)
    return release != nil
}

type Release struct {
    Name string         `json:"name"`
    Version string      `json:"version"`
    Namespace string    `json:"namespace"`
    Status string       `json:"status"`
    LastUpdated string  `json:"lastUpdated"`
}

//export go_helm_list_releases
//We are returning a JSON document as GoSlices (array) of objects was a nightmare to share between Go and Rust
func go_helm_list_releases(namespace string) *C.char {
    helmClient := getHelmClient(namespace, true)

    // List all releases, not only the deployed ones (e.g. include pending installations)
    releases, err := helmClient.ListReleasesByStateMask(action.ListAll)
    if err != nil {
        panic(err)
    }
    var result = make([]Release, len(releases))
    for i, release := range releases {
        result[i] = Release{
            Name: release.Name,
            Version: release.Chart.Metadata.Version,
            Namespace: release.Namespace,
            Status: release.Info.Status.String(),
            LastUpdated: release.Info.LastDeployed.String(),
        };
    }

    json, err := json.Marshal(result)
    if err != nil {
        panic(err)
    }

    return C.CString(string(json))
}

//export go_add_helm_repo
func go_add_helm_repo(name string, url string) {
    helmClient := getHelmClient("default", true) // Namespace doesn't matter

    chartRepo := repo.Entry{
        Name: name,
        URL:  url,
    }

    if err := helmClient.AddOrUpdateChartRepo(chartRepo); err != nil {
        panic(err)
    }
}

func getHelmClient(namespace string, suppressOutput bool) gohelm.Client {
    options := gohelm.Options {
        Namespace: namespace,
        Debug:     false,
    }

    if suppressOutput {
        options.DebugLog = func(format string, v ...interface{}) {}
    }

    helmClient, err := gohelm.New(&options)

    if err != nil {
        panic(err)
    }

    return helmClient
}
