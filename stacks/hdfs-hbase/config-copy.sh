#!/bin/bash

set -x

PROJECT_DIR=$(dirname $0)
NAMESPACE=${1}

mkdir $PROJECT_DIR/tmp

# get dependencies
cd $PROJECT_DIR/tmp
curl -O https://repo.stackable.tech/repository/misc/hbase-spark-k8s/hbase-spark-protocol-shaded-1.0.1-SNAPSHOT.jar
curl -O https://repo.stackable.tech/repository/misc/hbase-spark-k8s/original-hdfs-hbase-load-1.0.jar
curl -O https://repo.stackable.tech/repository/misc/hbase-spark-k8s/scala-library-2.12.14.jar
cd ..

# set envs
export HBASE-REGIONSERVER-0="hbase-regionserver-default-0"
export HBASE-REGIONSERVER-1="hbase-regionserver-default-1"

# copy jars to HBASE-REGIONSERVER-0
#kubectl -n ${NAMESPACE} cp tmp/hbase-spark-protocol-shaded-1.0.1-SNAPSHOT.jar $HBASE-REGIONSERVER-0:/stackable/hbase-2.4.12/lib
#kubectl -n ${NAMESPACE} cp tmp/hbase-spark-k8s/original-hdfs-hbase-load-1.0.jar $HBASE-REGIONSERVER-0:/stackable/hbase-2.4.12/lib
#kubectl -n ${NAMESPACE} cp tmp/hbase-spark-k8s/repository/misc/hbase-spark-k8s/scala-library-2.12.14.jar $HBASE-REGIONSERVER-0:/stackable/hbase-2.4.12/lib

# copy jars to HBASE-REGIONSERVER-1
#kubectl -n ${NAMESPACE} cp tmp/hbase-spark-protocol-shaded-1.0.1-SNAPSHOT.jar $HBASE-REGIONSERVER-1:/stackable/hbase-2.4.12/lib
#kubectl -n ${NAMESPACE} cp tmp/hbase-spark-k8s/original-hdfs-hbase-load-1.0.jar $HBASE-REGIONSERVER-1:/stackable/hbase-2.4.12/lib
#kubectl -n ${NAMESPACE} cp tmp/hbase-spark-k8s/repository/misc/hbase-spark-k8s/scala-library-2.12.14.jar $HBASE-REGIONSERVER-1:/stackable/hbase-2.4.12/lib


rm -r tmp