#!/bin/bash


kubectl -n stacks-demos cp 202004-divvy-tripdata.csv hdfs-namenode-default-0:/tmp

kubectl exec -n stacks-demos hdfs-namenode-default-0 -- /bin/bash -x -v -c "bin/hdfs dfs -put /tmp/row-key_losses_ukraine.csv /hbase"

echo 'create "cycling-tripdata","rideable_type","started_at","ended_at","start_station_name","start_station_id","end_station_name","end_station_id","start_lat","start_lng","end_lat","end_lng","member_casual"' | bin/hbase shell -n

/stackable/hbase/bin/hbase org.apache.hadoop.hbase.mapreduce.ImportTsv \
          -Dimporttsv.separator=, \
          -Dimporttsv.columns=HBASE_ROW_KEY,rideable_type,started_at,ended_at,start_station_name,start_station_id,end_station_name,end_station_id,start_lat,start_lng,end_lat,end_lng,member_casual \
          -Dimporttsv.bulk.output=hdfs://hdfs/hfile3 \
          cycling-tripdata hdfs://hdfs/data/raw/202004-divvy-tripdata.csv

