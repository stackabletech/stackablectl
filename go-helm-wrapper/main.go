package main

import (
	"C"
	"context"
	"time"
	"helm.sh/helm/v3/pkg/repo"
	gohelm "github.com/mittwald/go-helm-client"
)

func main() {
    go_add_helm_repo("stackable-stable", "https://repo.stackable.tech/repository/helm-stable/")

    if ! go_helm_release_exists("superset-operator") {
	    go_install_helm_release("superset-operator", "stackable-stable/superset-operator")
    }
}

//export go_install_helm_release
func go_install_helm_release(releaseName string, chartName string) {
// 	fmt.Printf("Installing helm release %s with chart URL %s\n", releaseName, chartName)
	helmClient := getHelmClient()

	timeout, _ := time.ParseDuration("10m")
	chartSpec := gohelm.ChartSpec{
		ReleaseName: releaseName,
		ChartName:   chartName,
		Namespace:   "default",
		UpgradeCRDs: true,
		Wait:        true,
		Timeout:     timeout,
	}

	if _, err := helmClient.InstallChart(context.Background(), &chartSpec); err != nil {
		panic(err)
	}
}

//export go_helm_release_exists
func go_helm_release_exists(releaseName string) bool {
// 	fmt.Printf("Checking if helm release %s exists\n", releaseName)
	helmClient := getHelmClient()

    release, _ := helmClient.GetRelease(releaseName)
    return release != nil
}

//export go_add_helm_repo
func go_add_helm_repo(name string, url string) {
// 	fmt.Printf("Adding helm repo %s with URL %s\n", name, url)
	helmClient := getHelmClient()

	chartRepo := repo.Entry{
		Name: name,
		URL:  url,
	}

	if err := helmClient.AddOrUpdateChartRepo(chartRepo); err != nil {
		panic(err)
	}
}

func getHelmClient() gohelm.Client {
	helmClient, err := gohelm.New(&gohelm.Options{
		Namespace: "default",
		Debug:     false,
	})

	if err != nil {
		panic(err)
	}

	return helmClient
}
