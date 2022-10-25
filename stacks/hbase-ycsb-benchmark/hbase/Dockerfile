# docker build . -t docker.stackable.tech/stackable/hbase:2.4.12-stackable0.2.0-jmx-conf
FROM docker.stackable.tech/stackable/hbase:2.4.12-stackable0.2.0

COPY jmx-config.yaml /stackable/jmx/region-server.yaml

USER 0
RUN sed -i '$d' /usr/lib/jvm/jre-*/conf/security/java.security
RUN sed -i '$d' /usr/lib/jvm/jre-*/conf/security/java.security

RUN sed -i '$d' /etc/java/java-*-openjdk/java-*/conf/security/java.security
RUN sed -i '$d' /etc/java/java-*-openjdk/java-*/conf/security/java.security
USER stackable
