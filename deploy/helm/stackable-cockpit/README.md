# Helm Chart for Stackable Cockpit

This Helm Chart can be used to install the Stackable Cockpit.

## Requirements

- Create a [Kubernetes Cluster](../Readme.md)
- Install [Helm](https://helm.sh/docs/intro/install/)

## Install the Stackable Operator for Stackable Cockpit

```bash
# From the root of the operator repository
make compile-chart

helm install stackable-cockpit deploy/helm/stackable-cockpit
```

## Links

[Source](https://github.com/stackabletech/stackablectl)
