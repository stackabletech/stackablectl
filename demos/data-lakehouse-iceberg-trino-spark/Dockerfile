# docker build . -t docker.stackable.tech/demos/pyspark-k8s-with-kafka-and-iceberg:3.3.0-stackable23.4

FROM docker.stackable.tech/stackable/pyspark-k8s:3.3.0-stackable23.4

RUN curl -L -O http://search.maven.org/remotecontent?filepath=org/apache/ivy/ivy/2.5.0/ivy-2.5.0.jar
RUN java -Divy.cache.dir=/tmp -Divy.home=/tmp -jar ivy-2.5.0.jar -notransitive \
-dependency org.apache.spark spark-sql-kafka-0-10_2.12 3.3.0 \
-retrieve "/stackable/spark/jars/[artifact]-[revision](-[classifier]).[ext]"
RUN java -Divy.cache.dir=/tmp -Divy.home=/tmp -jar ivy-2.5.0.jar -confs compile \
-dependency org.apache.spark spark-sql-kafka-0-10_2.12 3.3.0 \
-retrieve "/stackable/spark/jars/[artifact]-[revision](-[classifier]).[ext]"
RUN java -Divy.cache.dir=/tmp -Divy.home=/tmp -jar ivy-2.5.0.jar -notransitive \
-dependency org.apache.iceberg iceberg-spark-runtime-3.3_2.12 1.2.1 \
-retrieve "/stackable/spark/jars/[artifact]-[revision](-[classifier]).[ext]"
RUN java -Divy.cache.dir=/tmp -Divy.home=/tmp -jar ivy-2.5.0.jar -confs compile \
-dependency org.apache.iceberg iceberg-spark-runtime-3.3_2.12 1.2.1 \
-retrieve "/stackable/spark/jars/[artifact]-[revision](-[classifier]).[ext]"
