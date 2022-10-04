#!/bin/bash

set -x

PROJECT_DIR=$(dirname $0)
NAMESPACE=${1}


#mkdir $PROJECT_DIR/tmp

# get dependencies
#cd $PROJECT_DIR/tmp
#curl -O https://repo.stackable.tech/repository/misc/hbase-spark-k8s/hbase-spark-protocol-shaded-1.0.1-SNAPSHOT.jar
#curl -O https://repo.stackable.tech/repository/misc/hbase-spark-k8s/hbase-spark-1.0.1-SNAPSHOT.jar
#curl -O https://repo.stackable.tech/repository/misc/hbase-spark-k8s/scala-library-2.12.14.jar


# set envs
export HBASE_REGIONSERVER_0="hbase-regionserver-default-0"
export HBASE_REGIONSERVER_1="hbase-regionserver-default-1"

# copy jars to HBASE_REGIONSERVER_0
kubectl -n ${NAMESPACE} cp hbase-spark-protocol-shaded-1.0.1-SNAPSHOT.jar $HBASE_REGIONSERVER_0:/stackable/hbase-2.4.12/lib
kubectl -n ${NAMESPACE} cp hbase-spark-1.0.1-SNAPSHOT.jar $HBASE_REGIONSERVER_0:/stackable/hbase-2.4.12/lib
kubectl -n ${NAMESPACE} cp scala-library-2.12.14.jar $HBASE_REGIONSERVER_0:/stackable/hbase-2.4.12/lib

# copy jars to HBASE_REGIONSERVER_1
kubectl -n ${NAMESPACE} cp hbase-spark-protocol-shaded-1.0.1-SNAPSHOT.jar $HBASE_REGIONSERVER_1:/stackable/hbase-2.4.12/lib
kubectl -n ${NAMESPACE} cp hbase-spark-1.0.1-SNAPSHOT.jar $HBASE_REGIONSERVER_1:/stackable/hbase-2.4.12/lib
kubectl -n ${NAMESPACE} cp scala-library-2.12.14.jar $HBASE_REGIONSERVER_1:/stackable/hbase-2.4.12/lib

