# docker build . -t docker.stackable.tech/stackable/hadoop:3.3.4-stackable0.1.0-dns-fix
FROM docker.stackable.tech/stackable/hadoop:3.3.4-stackable0.1.0

USER 0
RUN sed -i '$d' /usr/lib/jvm/jre-*/conf/security/java.security
RUN sed -i '$d' /usr/lib/jvm/jre-*/conf/security/java.security

RUN sed -i '$d' /etc/java/java-*-openjdk/java-*/conf/security/java.security
RUN sed -i '$d' /etc/java/java-*-openjdk/java-*/conf/security/java.security
USER stackable
