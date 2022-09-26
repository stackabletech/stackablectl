package tech.stackable.demo.spark;

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
import org.apache.spark.api.java.function.Function;
import org.apache.spark.sql.SparkSession;
import scala.Tuple2;
import org.apache.log4j.LogManager;
import org.apache.log4j.Logger;

import java.util.List;

public class scan {

    private static final String CMD_HBASE_SITE = "hbaseSite";
    private static final String CMD_CORE_SITE = "coreSite";
    private static final String CMD_HDFS_SITE = "hdfsSite";

    private static final String CMD_TABLENAME = "tableName";

    private static final Logger LOGGER = LogManager.getLogger(scan.class);

    public static void main(String[] args) throws ParseException {

        final CommandLine commandLine = buildCommandLineParser(args);

        final String hbaseSite = String.valueOf(commandLine.getOptionValue(CMD_HBASE_SITE));
        final String coreSite = String.valueOf(commandLine.getOptionValue(CMD_CORE_SITE));
        final String hdfsSite = String.valueOf(commandLine.getOptionValue(CMD_HDFS_SITE));
        final String tableName = String.valueOf(commandLine.getOptionValue(CMD_TABLENAME));

        LOGGER.info("*** inputPath ***: " + tableName);
        LOGGER.info("*** hbaseSite ***: " + hbaseSite);
        LOGGER.info("*** coreSite ***: " + coreSite);
        LOGGER.info("*** hdfsSite ***: " + hdfsSite);

        Configuration config = HBaseConfiguration.create();
        config.addResource(new Path(hbaseSite));
        config.addResource(new Path(coreSite));
        config.addResource(new Path(hdfsSite));

        SparkSession spark = SparkSession.builder().appName("sparkHbase").getOrCreate();

        JavaSparkContext jsc = new JavaSparkContext(spark.sparkContext());
        // How to add HavaHbaseContext:
        // clone repo: https://github.com/apache/hbase-connectors/tree/master/spark
        // mvn -Dspark.version=3.3.0 -Dscala.version=2.12.14 -Dhadoop-three.version=3.3.2 -Dscala.binary.version=2.12 -Dhbase.version=2.4.12 -DrecompileMode=all -DskipTests clean package
        // Intellij: Project Structure --> add library --> New Library --> Java --> hbase-spark-1.0.1-SNAPSHOT.jar
        JavaHBaseContext hbaseContext = new JavaHBaseContext(jsc, config);

        try {
            Scan scan = new Scan();
            scan.setCaching(100);


            JavaRDD<Tuple2<ImmutableBytesWritable, Result>> javaRdd =
                    hbaseContext.hbaseRDD(TableName.valueOf(tableName), scan);

            List<String> results = javaRdd.map(new ScanConvertFunction()).collect();

            LOGGER.info("*** Result Size: ***: " + results.size());
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