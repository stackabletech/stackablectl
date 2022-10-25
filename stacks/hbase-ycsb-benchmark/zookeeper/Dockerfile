# docker build . -t docker.stackable.tech/stackable/zookeeper:3.8.0-stackable0.7.1-dns-fix
FROM docker.stackable.tech/stackable/zookeeper:3.8.0-stackable0.7.1

USER 0
RUN sed -i '$d' /usr/lib/jvm/jre-*/conf/security/java.security
RUN sed -i '$d' /usr/lib/jvm/jre-*/conf/security/java.security

RUN sed -i '$d' /etc/java/java-*-openjdk/java-*/conf/security/java.security
RUN sed -i '$d' /etc/java/java-*-openjdk/java-*/conf/security/java.security
USER stackable
