# Licensed to the Apache Software Foundation (ASF) under one or more
# contributor license agreements.  See the NOTICE file distributed with
# this work for additional information regarding copyright ownership.
# The ASF licenses this file to You under the Apache License, Version 2.0
# (the "License"); you may not use this file except in compliance with
# the License.  You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#

FROM ubuntu:22.04

ARG spark_version="3.3.1"
ARG hadoop_version="3"
ARG hadoop_long_version="3.2.2"
ARG spark_checksum="769db39a560a95fd88b58ed3e9e7d1e92fb68ee406689fb4d30c033cb5911e05c1942dcc70e5ec4585df84e80aabbc272b9386a208debda89522efff1335c8ff"
ARG openjdk_version="17"

ENV APACHE_SPARK_VERSION="${spark_version}" \
    HADOOP_VERSION="${hadoop_version}" \
    HADOOP_LONG_VERSION="${hadoop_long_version}" \
    SPARK_HOME=/usr/local/spark \
    JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64 \
    HOME=/stackable \
    SPARK_OPTS="--driver-java-options=-Xms1024M --driver-java-options=-Xmx4096M --driver-java-options=-Dlog4j.logLevel=info" \
    PATH="${PATH}:${SPARK_HOME}/bin:$HOME/app/bin"

RUN apt-get update --yes && \
    apt-get install --yes --no-install-recommends \
    "openjdk-${openjdk_version}-jre-headless" \
    ca-certificates-java \
    python3.10 python3-pip wget && \
    #python3.10-venv && \
    pip3 install --upgrade pip setuptools && \
    #python3 -m venv --system-site-packages /stackable/app && \
    #source /stackable/app/bin/activate && \
    #pip install --no-cache-dir --upgrade pip && \
    pip install --no-cache-dir scikit-learn==1.1.3 pandas==1.5.1 && \
    apt-get clean && rm -rf /var/lib/apt/lists/* \
    rm -r /root/.cache && rm -rf /var/cache/apt/*

RUN ln -s /usr/bin/python3 /usr/bin/python

WORKDIR /tmp

RUN wget -qO "spark.tgz" "https://archive.apache.org/dist/spark/spark-${APACHE_SPARK_VERSION}/spark-${APACHE_SPARK_VERSION}-bin-hadoop${HADOOP_VERSION}.tgz" && \
  echo "${spark_checksum} *spark.tgz" | sha512sum -c - && \
  tar xzf "spark.tgz" -C /usr/local --owner root --group root --no-same-owner && \
  rm "spark.tgz" && \
  ln -s /usr/local/spark-${APACHE_SPARK_VERSION}-bin-hadoop${HADOOP_VERSION} /usr/local/spark

RUN mkdir -p ${SPARK_HOME}/python && mkdir -p ${SPARK_HOME}/jars

# Add S3A support
RUN wget -O ${SPARK_HOME}/jars/hadoop-aws-${HADOOP_LONG_VERSION}.jar https://repo1.maven.org/maven2/org/apache/hadoop/hadoop-aws/${HADOOP_LONG_VERSION}/hadoop-aws-${HADOOP_LONG_VERSION}.jar
RUN wget -O ${SPARK_HOME}/jars/aws-java-sdk-bundle-1.12.270.jar https://repo1.maven.org/maven2/com/amazonaws/aws-java-sdk-bundle/1.12.270/aws-java-sdk-bundle-1.12.270.jar

RUN wget -O /usr/bin/tini https://repo.stackable.tech/repository/packages/tini/tini-x86_64
RUN chmod +x /usr/bin/tini

RUN groupadd -r stackable --gid=1000 && \
    useradd -r -g stackable --uid=1000 -d /stackable stackable && \
    chown -R stackable:stackable /stackable

USER stackable

WORKDIR /stackable
ENTRYPOINT [ "/usr/local/spark/kubernetes/dockerfiles/spark/entrypoint.sh" ]