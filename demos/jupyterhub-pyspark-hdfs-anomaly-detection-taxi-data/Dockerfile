# docker build -t docker.stackable.tech/demos/pyspark-k8s-with-scikit-learn:3.3.0-stackable23.4 .

FROM docker.stackable.tech/stackable/pyspark-k8s:3.3.0-stackable23.4

COPY requirements.txt .

RUN pip install --no-cache-dir --upgrade pip && \
    pip install --no-cache-dir -r ./requirements.txt
