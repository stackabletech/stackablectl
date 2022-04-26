package main

import (
    "C"
    "context"
    "encoding/json"
    "time"
    "helm.sh/helm/v3/pkg/repo"
    gohelm "github.com/mittwald/go-helm-client"
)

func main() {

}

//export go_install_helm_release
func go_install_helm_release(releaseName string, chartName string, chartVersion string, suppressOutput bool) {
    helmClient := getHelmClient(suppressOutput)

    timeout, _ := time.ParseDuration("10m")
    chartSpec := gohelm.ChartSpec{
        ReleaseName: releaseName,
        ChartName:   chartName,
        Version:     chartVersion,
        Namespace:   "default",
        UpgradeCRDs: true,
        Wait:        true,
        Timeout:     timeout,
    }

    if _, err := helmClient.InstallChart(context.Background(), &chartSpec); err != nil {
        panic(err)
    }
}

//export go_uninstall_helm_release
func go_uninstall_helm_release(releaseName string, suppressOutput bool) {
    helmClient := getHelmClient(suppressOutput)

    if err := helmClient.UninstallReleaseByName(releaseName); err != nil {
        panic(err)
    }
}

//export go_helm_release_exists
func go_helm_release_exists(releaseName string) bool {
    helmClient := getHelmClient(true)

    release, _ := helmClient.GetRelease(releaseName)
    return release != nil
}

type Release struct {
    Name string         `json:"name"`
    Version string      `json:"version"`
    Namespace string    `json:"namespace"`
    LastUpdated string  `json:"lastUpdated"`
}

//export go_helm_list_releases
//We are returning a JSON document as GoSlices (array) of objects was a nightmare to share between Go and Rust
func go_helm_list_releases() *C.char {
    helmClient := getHelmClient(true)

    releases, err := helmClient.ListDeployedReleases()
    if err != nil {
        panic(err)
    }
    var result = make([]Release, len(releases))
    for i, release := range releases {
        result[i] = Release{
            Name: release.Name,
            Version: release.Chart.Metadata.Version,
            Namespace: release.Namespace,
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
    helmClient := getHelmClient(true)

    chartRepo := repo.Entry{
        Name: name,
        URL:  url,
    }

    if err := helmClient.AddOrUpdateChartRepo(chartRepo); err != nil {
        panic(err)
    }
}

func getHelmClient(suppressOutput bool) gohelm.Client {
    options := gohelm.Options {
        Namespace: "default",
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
