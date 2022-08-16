package com.stackable.demo;


import org.apache.commons.cli.*;
import org.apache.hadoop.conf.Configuration;
import org.apache.hadoop.fs.Path;
import org.apache.hadoop.hbase.HBaseConfiguration;
import org.apache.hadoop.hbase.TableName;
import org.apache.hadoop.hbase.client.Result;
import org.apache.hadoop.hbase.client.Scan;
import org.apache.hadoop.hbase.io.ImmutableBytesWritable;
import org.apache.hadoop.hbase.spark.JavaHBaseContext;
import org.apache.hadoop.hbase.util.Bytes;
import org.apache.spark.api.java.JavaRDD;
import org.apache.spark.api.java.JavaSparkContext;
import org.apache.spark.sql.SparkSession;
import org.apache.spark.api.java.function.Function;
import scala.Tuple2;

import java.util.List;


final public class load {

    private load() {}

    private static final String CMD_HBASE_SITE = "hbaseSite";
    private static final String CMD_CORE_SITE = "coreSite";
    private static final String CMD_HDFS_SITE = "hdfsSite";

    private static final String CMD_TABLENAME = "tableName";



    public static void main(String[] args) throws ParseException {

        final CommandLine commandLine = buildCommandLineParser(args);

        final String hbaseSite = String.valueOf(commandLine.getOptionValue(CMD_HBASE_SITE));
        final String coreSite = String.valueOf(commandLine.getOptionValue(CMD_CORE_SITE));
        final String hdfsSite = String.valueOf(commandLine.getOptionValue(CMD_HDFS_SITE));
        final String tableName = String.valueOf(commandLine.getOptionValue(CMD_TABLENAME));

        Configuration config = HBaseConfiguration.create();
        config.addResource(new Path(hbaseSite));
        config.addResource(new Path(coreSite));
        config.addResource(new Path(hdfsSite));

        SparkSession spark = SparkSession.builder().appName("sparkHdfs").getOrCreate();

        JavaSparkContext jsc = new JavaSparkContext(spark.sparkContext());
        JavaHBaseContext hbaseContext = new JavaHBaseContext(jsc, config);

        try {
            Scan scan = new Scan();
            scan.setCaching(100);


            JavaRDD<Tuple2<ImmutableBytesWritable, Result>> javaRdd =
                    hbaseContext.hbaseRDD(TableName.valueOf(tableName), scan);

            List<String> results = javaRdd.map(new ScanConvertFunction()).collect();

            System.out.println("Result Size: " + results.size());
        } finally {
            jsc.stop();
        }
    }

        private static class ScanConvertFunction implements
            Function<Tuple2<ImmutableBytesWritable, Result>, String> {
            @Override
            public String call(Tuple2<ImmutableBytesWritable, Result> v1) throws Exception {
                return Bytes.toString(v1._1().copyBytes());
            }
        }


            static final CommandLine buildCommandLineParser(final String[] args) throws ParseException {
                final Options options = new Options();

                options.addOption(
                        OptionBuilder
                                .hasArg()
                                .withLongOpt(CMD_CORE_SITE)
                                .withArgName(CMD_CORE_SITE)
                                .withDescription("Config file for hdfs connection.")
                                .isRequired()
                                .create());

                options.addOption(
                        OptionBuilder
                                .hasArg()
                                .withLongOpt(CMD_HBASE_SITE)
                                .withArgName(CMD_HBASE_SITE)
                                .withDescription("Config file for zookeeper.")
                                .isRequired()
                                .create());

                options.addOption(
                        OptionBuilder
                                .hasArg()
                                .withLongOpt(CMD_HDFS_SITE)
                                .withArgName(CMD_HDFS_SITE)
                                .withDescription("Config file for HDFS.")
                                .isRequired()
                                .create());

                options.addOption(
                        OptionBuilder
                                .hasArg()
                                .withLongOpt(CMD_TABLENAME)
                                .withArgName(CMD_TABLENAME)
                                .withDescription("Name of table to scan")
                                .isRequired()
                                .create());


                final CommandLineParser parser = new BasicParser();

                return parser.parse(options, args);
            }
    }
