#!/bin/bash

bin/hbase org.apache.hadoop.hbase.mapreduce.ImportTsv \
          -Dimporttsv.separator=, \
          -Dimporttsv.columns=HBASE_ROW_KEY,ID:equipment,model,sub_model,manufacturer,losses_total,,abandoned,abandoned and burned,abandoned and destroyed,captured,captured and destroyed,damaged,damaged and abandoned,damaged and captured,damaged beyond economical repair,damaged by Forpost-R,damaged by Orion and captured,destroyed,destroyed by Forpost-R,destroyed by Orion,destroyed by loitering munition,scuttled to prevent capture by Russia,sunk,sunk but raised by Russia \
          -Dimporttsv.bulk.output=hdfs://hdfs-namenode-default-0:8020/hbase losses_ukraine_equipment /data/equipment/row-key_losses_ukraine.csv