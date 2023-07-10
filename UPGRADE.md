stackablectl release uninstall 23.1

kubectl replace -f https://raw.githubusercontent.com/stackabletech/airflow-operator/main/deploy/helm/airflow-operator/crds/crds.yaml
kubectl replace -f https://raw.githubusercontent.com/stackabletech/commons-operator/main/deploy/helm/commons-operator/crds/crds.yaml
kubectl replace -f https://raw.githubusercontent.com/stackabletech/druid-operator/main/deploy/helm/druid-operator/crds/crds.yaml
kubectl replace -f https://raw.githubusercontent.com/stackabletech/hbase-operator/main/deploy/helm/hbase-operator/crds/crds.yaml
kubectl replace -f https://raw.githubusercontent.com/stackabletech/hdfs-operator/main/deploy/helm/hdfs-operator/crds/crds.yaml
kubectl replace -f https://raw.githubusercontent.com/stackabletech/hive-operator/main/deploy/helm/hive-operator/crds/crds.yaml
kubectl replace -f https://raw.githubusercontent.com/stackabletech/kafka-operator/main/deploy/helm/kafka-operator/crds/crds.yaml
kubectl replace -f https://raw.githubusercontent.com/stackabletech/listener-operator/main/deploy/helm/listener-operator/crds/crds.yaml
kubectl replace -f https://raw.githubusercontent.com/stackabletech/nifi-operator/main/deploy/helm/nifi-operator/crds/crds.yaml
kubectl replace -f https://raw.githubusercontent.com/stackabletech/opa-operator/main/deploy/helm/opa-operator/crds/crds.yaml
kubectl replace -f https://raw.githubusercontent.com/stackabletech/secret-operator/main/deploy/helm/secret-operator/crds/crds.yaml
kubectl replace -f https://raw.githubusercontent.com/stackabletech/spark-k8s-operator/main/deploy/helm/spark-k8s-operator/crds/crds.yaml
kubectl replace -f https://raw.githubusercontent.com/stackabletech/superset-operator/main/deploy/helm/superset-operator/crds/crds.yaml
kubectl replace -f https://raw.githubusercontent.com/stackabletech/trino-operator/main/deploy/helm/trino-operator/crds/crds.yaml
kubectl replace -f https://raw.githubusercontent.com/stackabletech/zookeeper-operator/main/deploy/helm/zookeeper-operator/crds/crds.yaml

stackablectl release install dev
