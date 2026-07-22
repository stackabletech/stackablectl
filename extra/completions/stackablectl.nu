module completions {

  def "nu-complete stackablectl chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Command line tool to interact with the Stackable Data Platform
  export extern stackablectl [
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl operator chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl operator listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Interact with single operator instead of the full platform
  export extern "stackablectl operator" [
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl operator chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl operator listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl operator list output_type" [] {
    [ "plain" "table" "json" "yaml" ]
  }

  def "nu-complete stackablectl operator list chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl operator list listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # List available operators
  export extern "stackablectl operator list" [
    --output(-o): string@"nu-complete stackablectl operator list output_type"
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl operator list chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl operator list listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl operator describe output_type" [] {
    [ "plain" "table" "json" "yaml" ]
  }

  def "nu-complete stackablectl operator describe chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl operator describe listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Print out detailed operator information
  export extern "stackablectl operator describe" [
    --output(-o): string@"nu-complete stackablectl operator describe output_type"
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl operator describe chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl operator describe listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
    OPERATOR: string          # Operator to describe
  ]

  def "nu-complete stackablectl operator install cluster_type" [] {
    [ "kind" "minikube" ]
  }

  def "nu-complete stackablectl operator install chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl operator install listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Install one or more operators
  export extern "stackablectl operator install" [
    --operator-namespace: string # Namespace in the cluster used to deploy the operators
    --operator-ns: string     # Namespace in the cluster used to deploy the operators
    --cluster(-c): string@"nu-complete stackablectl operator install cluster_type" # Type of local cluster to use for testing
    --cluster-name: string    # Name of the local cluster
    --cluster-nodes: string   # Number of total nodes in the local cluster
    --cluster-cp-nodes: string # Number of control plane nodes in the local cluster
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl operator install chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl operator install listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
    ...OPERATORS: string      # Operator(s) to install
  ]

  def "nu-complete stackablectl operator uninstall chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl operator uninstall listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Uninstall one or more operators
  export extern "stackablectl operator uninstall" [
    --operator-namespace: string # Namespace in the cluster used to deploy the operators
    --operator-ns: string     # Namespace in the cluster used to deploy the operators
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl operator uninstall chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl operator uninstall listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
    ...operators: string      # One or more operators to uninstall
  ]

  def "nu-complete stackablectl operator installed output_type" [] {
    [ "plain" "table" "json" "yaml" ]
  }

  def "nu-complete stackablectl operator installed chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl operator installed listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # List installed operators
  export extern "stackablectl operator installed" [
    --output(-o): string@"nu-complete stackablectl operator installed output_type"
    --operator-namespace: string # Namespace in the cluster used to deploy the operators
    --operator-ns: string     # Namespace in the cluster used to deploy the operators
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl operator installed chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl operator installed listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl operator help" [
  ]

  # List available operators
  export extern "stackablectl operator help list" [
  ]

  # Print out detailed operator information
  export extern "stackablectl operator help describe" [
  ]

  # Install one or more operators
  export extern "stackablectl operator help install" [
  ]

  # Uninstall one or more operators
  export extern "stackablectl operator help uninstall" [
  ]

  # List installed operators
  export extern "stackablectl operator help installed" [
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl operator help help" [
  ]

  def "nu-complete stackablectl release chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl release listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Interact with all operators of the platform which are released together
  export extern "stackablectl release" [
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl release chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl release listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl release list output_type" [] {
    [ "plain" "table" "json" "yaml" ]
  }

  def "nu-complete stackablectl release list chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl release list listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # List available releases
  export extern "stackablectl release list" [
    --output(-o): string@"nu-complete stackablectl release list output_type"
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl release list chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl release list listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl release describe output_type" [] {
    [ "plain" "table" "json" "yaml" ]
  }

  def "nu-complete stackablectl release describe chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl release describe listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Print out detailed release information
  export extern "stackablectl release describe" [
    --output(-o): string@"nu-complete stackablectl release describe output_type"
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl release describe chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl release describe listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
    RELEASE: string
  ]

  def "nu-complete stackablectl release install cluster_type" [] {
    [ "kind" "minikube" ]
  }

  def "nu-complete stackablectl release install chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl release install listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Install a specific release
  export extern "stackablectl release install" [
    --include(-i): string     # Whitelist of product operators to install
    --exclude(-e): string     # Blacklist of product operators to install
    --operator-namespace: string # Namespace in the cluster used to deploy the operators
    --operator-ns: string     # Namespace in the cluster used to deploy the operators
    --cluster(-c): string@"nu-complete stackablectl release install cluster_type" # Type of local cluster to use for testing
    --cluster-name: string    # Name of the local cluster
    --cluster-nodes: string   # Number of total nodes in the local cluster
    --cluster-cp-nodes: string # Number of control plane nodes in the local cluster
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl release install chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl release install listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
    RELEASE: string           # Release to install
  ]

  def "nu-complete stackablectl release uninstall chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl release uninstall listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Uninstall a release
  export extern "stackablectl release uninstall" [
    --operator-namespace: string # Namespace in the cluster used to deploy the operators
    --operator-ns: string     # Namespace in the cluster used to deploy the operators
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl release uninstall chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl release uninstall listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
    RELEASE: string           # Name of the release to uninstall
  ]

  def "nu-complete stackablectl release upgrade chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl release upgrade listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Upgrade a release
  export extern "stackablectl release upgrade" [
    --include(-i): string     # List of product operators to upgrade
    --exclude(-e): string     # Blacklist of product operators to install
    --operator-namespace: string # Namespace in the cluster used to deploy the operators
    --operator-ns: string     # Namespace in the cluster used to deploy the operators
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl release upgrade chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl release upgrade listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
    RELEASE: string           # Upgrade to the specified release
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl release help" [
  ]

  # List available releases
  export extern "stackablectl release help list" [
  ]

  # Print out detailed release information
  export extern "stackablectl release help describe" [
  ]

  # Install a specific release
  export extern "stackablectl release help install" [
  ]

  # Uninstall a release
  export extern "stackablectl release help uninstall" [
  ]

  # Upgrade a release
  export extern "stackablectl release help upgrade" [
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl release help help" [
  ]

  def "nu-complete stackablectl stack chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl stack listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Interact with stacks, which are ready-to-use product combinations
  export extern "stackablectl stack" [
    --release: string         # Target a specific Stackable release
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl stack chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl stack listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl stack list output_type" [] {
    [ "plain" "table" "json" "yaml" ]
  }

  def "nu-complete stackablectl stack list chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl stack list listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # List available stacks
  export extern "stackablectl stack list" [
    --output(-o): string@"nu-complete stackablectl stack list output_type"
    --release: string         # Target a specific Stackable release
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl stack list chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl stack list listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl stack describe output_type" [] {
    [ "plain" "table" "json" "yaml" ]
  }

  def "nu-complete stackablectl stack describe chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl stack describe listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Describe a specific stack
  export extern "stackablectl stack describe" [
    --output(-o): string@"nu-complete stackablectl stack describe output_type"
    --release: string         # Target a specific Stackable release
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl stack describe chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl stack describe listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
    stack_name: string        # Name of the stack to describe
  ]

  def "nu-complete stackablectl stack install cluster_type" [] {
    [ "kind" "minikube" ]
  }

  def "nu-complete stackablectl stack install chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl stack install listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Install a specific stack
  export extern "stackablectl stack install" [
    --skip-release            # Skip the installation of the release during the stack install process
    --stack-parameters: string # List of parameters to use when installing the stack
    --parameters: string      # List of parameters to use when installing the stack
    --cluster(-c): string@"nu-complete stackablectl stack install cluster_type" # Type of local cluster to use for testing
    --cluster-name: string    # Name of the local cluster
    --cluster-nodes: string   # Number of total nodes in the local cluster
    --cluster-cp-nodes: string # Number of control plane nodes in the local cluster
    --operator-namespace: string # Namespace where the operators are deployed
    --operator-ns: string     # Namespace where the operators are deployed
    --namespace(-n): string   # Namespace where the stacks or demos are deployed
    --product-ns: string      # Namespace where the stacks or demos are deployed
    --assume-yes(-y)          # Assume "yes" as answer to all prompts and run non-interactively
    --yes                     # Assume "yes" as answer to all prompts and run non-interactively
    --release: string         # Target a specific Stackable release
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl stack install chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl stack install listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
    stack_name: string        # Name of the stack to install
  ]

  def "nu-complete stackablectl stack uninstall chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl stack uninstall listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Uninstall a specific stack. Caution: This will delete the provided stack namespace, the operators and provided operator namespace, and all Stackable CRDs
  export extern "stackablectl stack uninstall" [
    --operator-namespace: string # Namespace where the operators are deployed
    --operator-ns: string     # Namespace where the operators are deployed
    --namespace(-n): string   # Namespace where the stacks or demos are deployed
    --product-ns: string      # Namespace where the stacks or demos are deployed
    --skip-operators-and-crds # Skip uninstalling Stackable operators and CRDs
    --assume-yes(-y)          # Assume "yes" as answer to all prompts and run non-interactively
    --yes                     # Assume "yes" as answer to all prompts and run non-interactively
    --release: string         # Target a specific Stackable release
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl stack uninstall chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl stack uninstall listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
    stack_name: string        # Name of the stack to uninstall
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl stack help" [
  ]

  # List available stacks
  export extern "stackablectl stack help list" [
  ]

  # Describe a specific stack
  export extern "stackablectl stack help describe" [
  ]

  # Install a specific stack
  export extern "stackablectl stack help install" [
  ]

  # Uninstall a specific stack. Caution: This will delete the provided stack namespace, the operators and provided operator namespace, and all Stackable CRDs
  export extern "stackablectl stack help uninstall" [
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl stack help help" [
  ]

  def "nu-complete stackablectl stacklet chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl stacklet listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Interact with deployed stacklets, which are bundles of resources and containers required to run the product
  export extern "stackablectl stacklet" [
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl stacklet chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl stacklet listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl stacklet credentials chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl stacklet credentials listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Display credentials for a stacklet
  export extern "stackablectl stacklet credentials" [
    --namespace(-n): string   # Namespace in the cluster used to deploy the products
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl stacklet credentials chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl stacklet credentials listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
    product_name: string      # The name of the product, for example 'superset'
    stacklet_name: string     # The name of the stacklet, for example 'superset'
  ]

  def "nu-complete stackablectl stacklet list output_type" [] {
    [ "plain" "table" "json" "yaml" ]
  }

  def "nu-complete stackablectl stacklet list chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl stacklet list listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # List deployed stacklets
  export extern "stackablectl stacklet list" [
    --output(-o): string@"nu-complete stackablectl stacklet list output_type"
    --operator-namespace: string # Namespace where the operators are deployed
    --operator-ns: string     # Namespace where the operators are deployed
    --namespace(-n): string   # Namespace where the stacks or demos are deployed
    --product-ns: string      # Namespace where the stacks or demos are deployed
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl stacklet list chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl stacklet list listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl stacklet help" [
  ]

  # Display credentials for a stacklet
  export extern "stackablectl stacklet help credentials" [
  ]

  # List deployed stacklets
  export extern "stackablectl stacklet help list" [
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl stacklet help help" [
  ]

  def "nu-complete stackablectl demo chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl demo listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Interact with demos, which are end-to-end usage demonstrations of the Stackable data platform
  export extern "stackablectl demo" [
    --release: string         # Target a specific Stackable release
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl demo chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl demo listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl demo list output_type" [] {
    [ "plain" "table" "json" "yaml" ]
  }

  def "nu-complete stackablectl demo list chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl demo list listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # List available demos
  export extern "stackablectl demo list" [
    --output(-o): string@"nu-complete stackablectl demo list output_type"
    --release: string         # Target a specific Stackable release
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl demo list chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl demo list listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl demo describe output_type" [] {
    [ "plain" "table" "json" "yaml" ]
  }

  def "nu-complete stackablectl demo describe chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl demo describe listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Print out detailed demo information
  export extern "stackablectl demo describe" [
    --output(-o): string@"nu-complete stackablectl demo describe output_type"
    --release: string         # Target a specific Stackable release
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl demo describe chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl demo describe listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
    DEMO: string              # Demo to describe
  ]

  def "nu-complete stackablectl demo install cluster_type" [] {
    [ "kind" "minikube" ]
  }

  def "nu-complete stackablectl demo install chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl demo install listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Install a specific demo
  export extern "stackablectl demo install" [
    --skip-release            # Skip the installation of the release during the stack install process
    --stack-parameters: string # List of parameters to use when installing the stack
    --parameters: string      # List of parameters to use when installing the demo
    --cluster(-c): string@"nu-complete stackablectl demo install cluster_type" # Type of local cluster to use for testing
    --cluster-name: string    # Name of the local cluster
    --cluster-nodes: string   # Number of total nodes in the local cluster
    --cluster-cp-nodes: string # Number of control plane nodes in the local cluster
    --operator-namespace: string # Namespace where the operators are deployed
    --operator-ns: string     # Namespace where the operators are deployed
    --namespace(-n): string   # Namespace where the stacks or demos are deployed
    --product-ns: string      # Namespace where the stacks or demos are deployed
    --assume-yes(-y)          # Assume "yes" as answer to all prompts and run non-interactively
    --yes                     # Assume "yes" as answer to all prompts and run non-interactively
    --release: string         # Target a specific Stackable release
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl demo install chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl demo install listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
    DEMO: string              # Demo to install
  ]

  def "nu-complete stackablectl demo uninstall chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl demo uninstall listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Uninstall a specific stack. Caution: This will delete the provided stack namespace, the operators and provided operator namespace, and all Stackable CRDs
  export extern "stackablectl demo uninstall" [
    --operator-namespace: string # Namespace where the operators are deployed
    --operator-ns: string     # Namespace where the operators are deployed
    --namespace(-n): string   # Namespace where the stacks or demos are deployed
    --product-ns: string      # Namespace where the stacks or demos are deployed
    --skip-operators-and-crds # Skip uninstalling Stackable operators and CRDs
    --assume-yes(-y)          # Assume "yes" as answer to all prompts and run non-interactively
    --yes                     # Assume "yes" as answer to all prompts and run non-interactively
    --release: string         # Target a specific Stackable release
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl demo uninstall chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl demo uninstall listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
    demo_name: string         # Demo to uninstall
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl demo help" [
  ]

  # List available demos
  export extern "stackablectl demo help list" [
  ]

  # Print out detailed demo information
  export extern "stackablectl demo help describe" [
  ]

  # Install a specific demo
  export extern "stackablectl demo help install" [
  ]

  # Uninstall a specific stack. Caution: This will delete the provided stack namespace, the operators and provided operator namespace, and all Stackable CRDs
  export extern "stackablectl demo help uninstall" [
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl demo help help" [
  ]

  def "nu-complete stackablectl completions chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl completions listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Generate shell completions for this tool
  export extern "stackablectl completions" [
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl completions chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl completions listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl completions bash chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl completions bash listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Generate shell completions for Bash
  export extern "stackablectl completions bash" [
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl completions bash chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl completions bash listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl completions elvish chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl completions elvish listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Generate shell completions for Elvish
  export extern "stackablectl completions elvish" [
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl completions elvish chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl completions elvish listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl completions fish chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl completions fish listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Generate shell completions for Fish
  export extern "stackablectl completions fish" [
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl completions fish chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl completions fish listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl completions nushell chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl completions nushell listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Generate shell completions for Nushell
  export extern "stackablectl completions nushell" [
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl completions nushell chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl completions nushell listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl completions zsh chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl completions zsh listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Generate shell completions for ZSH
  export extern "stackablectl completions zsh" [
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl completions zsh chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl completions zsh listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl completions help" [
  ]

  # Generate shell completions for Bash
  export extern "stackablectl completions help bash" [
  ]

  # Generate shell completions for Elvish
  export extern "stackablectl completions help elvish" [
  ]

  # Generate shell completions for Fish
  export extern "stackablectl completions help fish" [
  ]

  # Generate shell completions for Nushell
  export extern "stackablectl completions help nushell" [
  ]

  # Generate shell completions for ZSH
  export extern "stackablectl completions help zsh" [
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl completions help help" [
  ]

  def "nu-complete stackablectl cache chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl cache listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Interact with locally cached files
  export extern "stackablectl cache" [
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl cache chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl cache listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl cache list chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl cache list listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # List cached files
  export extern "stackablectl cache list" [
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl cache list chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl cache list listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl cache clean chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl cache clean listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Clean cached files
  export extern "stackablectl cache clean" [
    --old                     # Only remove outdated files in the cache
    --outdated                # Only remove outdated files in the cache
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl cache clean chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl cache clean listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl cache help" [
  ]

  # List cached files
  export extern "stackablectl cache help list" [
  ]

  # Clean cached files
  export extern "stackablectl cache help clean" [
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl cache help help" [
  ]

  def "nu-complete stackablectl experimental-debug chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl experimental-debug listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # EXPERIMENTAL: Launch a debug container for a Pod
  export extern "stackablectl experimental-debug" [
    --namespace(-n): string   # The namespace of the Pod being debugged
    --container(-c): string   # The target container to debug
    --image: string           # The debug container image
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl experimental-debug chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl experimental-debug listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
    pod: string               # The Pod to debug
  ]

  def "nu-complete stackablectl version chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl version listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Retrieve version data of the stackablectl installation
  export extern "stackablectl version" [
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl version chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl version listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  def "nu-complete stackablectl version check chart_source" [] {
    [ "oci" "repo" ]
  }

  def "nu-complete stackablectl version check listener_class_preset" [] {
    [ "none" "stable-nodes" "ephemeral-nodes" ]
  }

  # Check if there is a new version available
  export extern "stackablectl version check" [
    --log-level(-l): string   # Log level this application uses
    --no-cache                # Do not cache the remote (default) demo, stack and release files
    --demo-file(-d): path     # Provide one or more additional (custom) demo file(s)
    --stack-file(-s): path    # Provide one or more additional (custom) stack file(s)
    --release-file(-r): path  # Provide one or more additional (custom) release file(s)
    --operator-values(-f): path # Path to a Helm values file that will be used for the installation of operators
    --helm-repo-stable: string # Provide a custom Helm stable repository URL
    --helm-repo-test: string  # Provide a custom Helm test repository URL
    --helm-repo-dev: string   # Provide a custom Helm dev repository URL
    --chart-source: string@"nu-complete stackablectl version check chart_source" # Source the charts from either a OCI registry or from index.yaml-based repositories
    --listener-class-preset: string@"nu-complete stackablectl version check listener_class_preset" # Choose the ListenerClass preset (`none`, `ephemeral-nodes` or `stable-nodes`)
    --help(-h)                # Print help (see more with '--help')
    --version(-V)             # Print version
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl version help" [
  ]

  # Check if there is a new version available
  export extern "stackablectl version help check" [
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl version help help" [
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl help" [
  ]

  # Interact with single operator instead of the full platform
  export extern "stackablectl help operator" [
  ]

  # List available operators
  export extern "stackablectl help operator list" [
  ]

  # Print out detailed operator information
  export extern "stackablectl help operator describe" [
  ]

  # Install one or more operators
  export extern "stackablectl help operator install" [
  ]

  # Uninstall one or more operators
  export extern "stackablectl help operator uninstall" [
  ]

  # List installed operators
  export extern "stackablectl help operator installed" [
  ]

  # Interact with all operators of the platform which are released together
  export extern "stackablectl help release" [
  ]

  # List available releases
  export extern "stackablectl help release list" [
  ]

  # Print out detailed release information
  export extern "stackablectl help release describe" [
  ]

  # Install a specific release
  export extern "stackablectl help release install" [
  ]

  # Uninstall a release
  export extern "stackablectl help release uninstall" [
  ]

  # Upgrade a release
  export extern "stackablectl help release upgrade" [
  ]

  # Interact with stacks, which are ready-to-use product combinations
  export extern "stackablectl help stack" [
  ]

  # List available stacks
  export extern "stackablectl help stack list" [
  ]

  # Describe a specific stack
  export extern "stackablectl help stack describe" [
  ]

  # Install a specific stack
  export extern "stackablectl help stack install" [
  ]

  # Uninstall a specific stack. Caution: This will delete the provided stack namespace, the operators and provided operator namespace, and all Stackable CRDs
  export extern "stackablectl help stack uninstall" [
  ]

  # Interact with deployed stacklets, which are bundles of resources and containers required to run the product
  export extern "stackablectl help stacklet" [
  ]

  # Display credentials for a stacklet
  export extern "stackablectl help stacklet credentials" [
  ]

  # List deployed stacklets
  export extern "stackablectl help stacklet list" [
  ]

  # Interact with demos, which are end-to-end usage demonstrations of the Stackable data platform
  export extern "stackablectl help demo" [
  ]

  # List available demos
  export extern "stackablectl help demo list" [
  ]

  # Print out detailed demo information
  export extern "stackablectl help demo describe" [
  ]

  # Install a specific demo
  export extern "stackablectl help demo install" [
  ]

  # Uninstall a specific stack. Caution: This will delete the provided stack namespace, the operators and provided operator namespace, and all Stackable CRDs
  export extern "stackablectl help demo uninstall" [
  ]

  # Generate shell completions for this tool
  export extern "stackablectl help completions" [
  ]

  # Generate shell completions for Bash
  export extern "stackablectl help completions bash" [
  ]

  # Generate shell completions for Elvish
  export extern "stackablectl help completions elvish" [
  ]

  # Generate shell completions for Fish
  export extern "stackablectl help completions fish" [
  ]

  # Generate shell completions for Nushell
  export extern "stackablectl help completions nushell" [
  ]

  # Generate shell completions for ZSH
  export extern "stackablectl help completions zsh" [
  ]

  # Interact with locally cached files
  export extern "stackablectl help cache" [
  ]

  # List cached files
  export extern "stackablectl help cache list" [
  ]

  # Clean cached files
  export extern "stackablectl help cache clean" [
  ]

  # EXPERIMENTAL: Launch a debug container for a Pod
  export extern "stackablectl help experimental-debug" [
  ]

  # Retrieve version data of the stackablectl installation
  export extern "stackablectl help version" [
  ]

  # Check if there is a new version available
  export extern "stackablectl help version check" [
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "stackablectl help help" [
  ]

}

export use completions *
