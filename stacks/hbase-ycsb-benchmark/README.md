Their env: 13 Data Nodes 251GB RAM / 48 & 72 Cores / Max Heap: 20G / 161 Tabellen, 12 HDD per Node

Important:
* We used europe-west1-d
* Enabled `Compact Placement`
** Now we can only use A2, C2, C2D, N2 and N2D machine types
* We also enable SSD persitent disk with 100gb capacity for every node
* We also enable `NodeLocal DNSCache`. As the jvm caches dns already this *should* make no difference but better safe than sorry
* After spinng up the cluster taint 1 node with `kubectl taint nodes gke-snocke-sbernauer-hba-default-pool-c252127f-zdfk app=ycsb:NoSchedule`
* `stackablectl operator install commons secret zookeeper hdfs hbase`
* `kubectl apply -f stacks/hbase-ycsb-benchmark/zookeeper.yaml # Wait until ready`
* `kubectl apply -f stacks/hbase-ycsb-benchmark/hdfs.yaml # Wait until ready`
* `kubectl apply -f stacks/hbase-ycsb-benchmark/hbase.yaml # Wait until ready`
* `kubectl apply -f stacks/hbase-ycsb-benchmark/ycsb.yaml # Wait until ready`


Nodepool with 16 nodes and `Compact Placement`:
```
Type            RAM   Price $ / hour
n2-standard-8   32g   4.80  # Intel
n2d-standard-8  32g   4.51  # AMD
c2-standard-8   32g   5.53  # Intel
c2d-standard-8  32g   6.00  # AMD
c2d-standard-16 64g   11.81 # AMD
```

`gcloud beta container --project "engineering-329019" clusters create "snocke-sbernauer-hbase-perf" --zone "europe-west1-d" --no-enable-basic-auth --cluster-version "1.23.8-gke.1900" --release-channel "regular" --machine-type "c2-standard-8" --placement-type COMPACT --image-type "COS_CONTAINERD" --disk-type "pd-ssd" --disk-size "100" --metadata disable-legacy-endpoints=true --scopes "https://www.googleapis.com/auth/devstorage.read_only","https://www.googleapis.com/auth/logging.write","https://www.googleapis.com/auth/monitoring","https://www.googleapis.com/auth/servicecontrol","https://www.googleapis.com/auth/service.management.readonly","https://www.googleapis.com/auth/trace.append" --max-pods-per-node "110" --num-nodes "16" --logging=SYSTEM,WORKLOAD --monitoring=SYSTEM --enable-ip-alias --network "projects/engineering-329019/global/networks/default" --subnetwork "projects/engineering-329019/regions/europe-west1/subnetworks/default" --no-enable-intra-node-visibility --default-max-pods-per-node "110" --no-enable-master-authorized-networks --addons HorizontalPodAutoscaling,HttpLoadBalancing,NodeLocalDNS,GcePersistentDiskCsiDriver --enable-autoupgrade --enable-autorepair --max-surge-upgrade 1 --max-unavailable-upgrade 0 --enable-shielded-nodes --node-locations "europe-west1-d"`

`create 'sbernauer', {NAME => 'benchmark', IN_MEMORY => false, TTL => 2147483647 , BLOOMFILTER => 'NONE', COMPRESSION => 'SNAPPY'}, SPLITS=> ['user1000000000000000000', 'user2000000000000000000', 'user3000000000000000000', 'user4000000000000000000', 'user5000000000000000000', 'user6000000000000000000', 'user7000000000000000000', 'user8000000000000000000', 'user9000000000000000000']`

`/stackable/ycsb-0.17.0/bin/ycsb.sh load hbase20 -p table=sbernauer -p columnfamily=benchmark -p recordcount=3000000 -p fieldcount=10 -p fieldlength=100 -p workload=site.ycsb.workloads.CoreWorkload -threads 10 -s`


`/stackable/ycsb-0.17.0/bin/ycsb.sh run hbase20 -p operationcount=3000000 -p table=sbernauer -p columnfamily=benchmark -P /stackable/ycsb-0.17.0//workloads/workloada -s -threads 100 -target 50000`
