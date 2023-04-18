# docker build -t docker.stackable.tech/demos/jupyter:pyspark-3.3.0-java17-stackable23.4.0-rc1 -f Dockerfile .

FROM docker.stackable.tech/stackable/pyspark-k8s:3.3.0-stackable23.4

COPY requirements.txt .

RUN pip install --no-cache-dir --upgrade pip && \
    pip install --no-cache-dir -r ./requirements.txt
