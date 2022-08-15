package com.stackable.demo;


import org.apache.commons.cli.*;
import org.apache.hadoop.conf.Configuration;
import org.apache.hadoop.fs.Path;
import org.apache.hadoop.hbase.HBaseConfiguration;
import org.apache.spark.api.java.JavaSparkContext;
import org.apache.spark.sql.SparkSession;

public class load {

    private static final String CMD_HBASE_SITE = "hbaseSite";
    private static final String CMD_CORE_SITE = "coreSite";
    private static final String CMD_HDFS_SITE = "hdfsSite";

    public static void main(String[] args) throws ParseException {

        final CommandLine commandLine = buildCommandLineParser(args);

        final String hbaseSite = String.valueOf(commandLine.getOptionValue(CMD_HBASE_SITE));
        final String coreSite = String.valueOf(commandLine.getOptionValue(CMD_CORE_SITE));
        final String hdfsSite = String.valueOf(commandLine.getOptionValue(CMD_HDFS_SITE));

        Configuration config = HBaseConfiguration.create();
        config.addResource(new Path(hbaseSite));
        config.addResource(new Path(coreSite));
        config.addResource(new Path(hdfsSite));

        SparkSession spark = SparkSession.builder().appName("sparkHdfs").getOrCreate();


        JavaSparkContext jsc = new JavaSparkContext(spark.sparkContext());
        JavaHBaseContext hbaseContext = new JavaHBaseContext(jsc, conf);

        // Instantiate HBaseContext
        new HBaseContext(spark.sparkContext());

        hbaseDF = (spark.read.format("org.apache.hadoop.hbase.spark")
                .option("hbase.columns.mapping",
                        "rowKey STRING :key," +
                                "firstName STRING Name:First, lastName STRING Name:Last," +
                                "country STRING Address:Country, state STRING Address:State"
                )
                .option("hbase.table", "Person")
        ).load()


x
    }

    static final CommandLine buildCommandLineParser(final String[] args) throws ParseException
    {
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


        final CommandLineParser parser = new BasicParser();

        return parser.parse(options, args);
}
