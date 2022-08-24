package com.stackable.demo;

import org.apache.commons.cli.*;
import org.apache.hadoop.conf.Configuration;
import org.apache.hadoop.fs.FileSystem;
import org.apache.hadoop.fs.Path;
import org.apache.hadoop.hbase.HBaseConfiguration;
import org.apache.hadoop.hbase.MasterNotRunningException;
import org.apache.hadoop.hbase.TableName;
import org.apache.hadoop.hbase.client.*;
import org.apache.hadoop.hbase.util.Bytes;
import org.apache.log4j.LogManager;
import org.apache.log4j.Logger;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.UUID;
import java.util.stream.Collectors;


public class hbaseTest {
    private static final TableName TABLE_NAME = TableName.valueOf("wines");
    private static final byte[] COLUMN_FAMILY_NAME = Bytes.toBytes("description");
    private static final byte[] COLUMN = Bytes.toBytes("country");

    private static final String CMD_INPUT = "input";
    private static final String CMD_TARGET_TABLE = "targetTable";
    private static final String CMD_HBASE_SITE = "hbaseSite";
    private static final String CMD_CORE_SITE = "coreSite";
    private static final String CMD_HDFS_SITE = "hdfsSite";

    private static final Logger LOGGER = LogManager.getLogger(hbaseTest.class);

    public static void createTable(final Admin admin) throws IOException {
        if(!admin.tableExists(TABLE_NAME)) {

            TableDescriptor desc = TableDescriptorBuilder.newBuilder(TABLE_NAME)
                    .setColumnFamily(ColumnFamilyDescriptorBuilder.of(COLUMN_FAMILY_NAME))
                    .build();

            admin.createTable(desc);
        }
    }

    public static List<String> readData(final Path path, Configuration configuration) throws IOException {
            FileSystem fs = FileSystem.get(configuration);
            BufferedReader reader = new BufferedReader(new InputStreamReader(fs.open(path)));

            return reader.lines().collect(Collectors.toList());
    }

    public static void main(String[] args) throws IOException, ParseException {

        // parse input parameters
        final CommandLine commandLine = buildCommandLineParser(args);

        final String inputPath = String.valueOf(commandLine.getOptionValue(CMD_INPUT));
        final String targetTable = String.valueOf(commandLine.getOptionValue(CMD_TARGET_TABLE));
        final String hbaseSite = String.valueOf(commandLine.getOptionValue(CMD_HBASE_SITE));
        final String coreSite = String.valueOf(commandLine.getOptionValue(CMD_CORE_SITE));
        final String hdfsSite = String.valueOf(commandLine.getOptionValue(CMD_HDFS_SITE));

        LOGGER.info("*** inputPath ***: " + inputPath);
        LOGGER.info("*** targetTable ***: " + targetTable);
        LOGGER.info("*** hbaseSite ***: " + hbaseSite);
        LOGGER.info("*** coreSite ***: " + coreSite);
        LOGGER.info("*** hdfsSite ***: " + hdfsSite);

        Configuration config = HBaseConfiguration.create();
        config.addResource(new Path(hbaseSite));
        config.addResource(new Path(coreSite));
        config.addResource(new Path(hdfsSite));
        // Tells the HbaseConfiguration class to use hdfs for filesystem
        config.set("fs.hdfs.impl", "org.apache.hadoop.hdfs.DistributedFileSystem");

        //config.writeXml(System.out);
        config.set("hbase.table.name", targetTable);

        try {
            HBaseAdmin.available(config);
        } catch (MasterNotRunningException e) {
            System.out.println("HBase is not running." + e.getMessage());
            return;
        }

        try (Connection connection = ConnectionFactory.createConnection(config); Admin admin = connection.getAdmin()) {
            Path path = new Path(inputPath);
            LOGGER.info("*** creating table ***: " + TABLE_NAME);
            createTable(admin);

            try(Table table = connection.getTable(TABLE_NAME)) {
                List<String> list = readData(path, config);
                LOGGER.info("*** Array Size ***: " + list.size());

                list.stream()
                        .forEach(e ->
                        {
                            try {
                                String rowId = UUID.randomUUID().toString();
                                table.put(new Put(rowId.getBytes(StandardCharsets.UTF_8))
                                        .addColumn(COLUMN_FAMILY_NAME, COLUMN, e.getBytes(StandardCharsets.UTF_8)));
                            } catch (IOException ex) {
                                throw new RuntimeException(ex);
                            }
                        });
            }
        }
    }

    static final CommandLine buildCommandLineParser(final String[] args) throws ParseException
    {
        final Options options = new Options();

        options.addOption(
                OptionBuilder
                        .hasArg()
                        .withLongOpt(CMD_INPUT)
                        .withArgName(CMD_INPUT)
                        .withDescription("HDFS input path.")
                        .isRequired()
                        .create());

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
                        .withLongOpt(CMD_TARGET_TABLE)
                        .withArgName(CMD_TARGET_TABLE)
                        .withDescription("Target table name in hbase.")
                        .isRequired()
                        .create());

        final CommandLineParser parser = new BasicParser();

        return parser.parse(options, args);

    }
}