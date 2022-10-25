package org.example

import org.apache.spark.sql.SparkSession
import org.apache.spark.sql.types.StructType
import com.linkedin.relevance.isolationforest.IsolationForest
import org.apache.spark.ml.feature.VectorAssembler
import org.apache.spark.sql.functions.{count, _}
import org.apache.spark.sql.types._
import org.apache.spark.sql.functions.{col, expr, to_timestamp}

/**
 * @author ${user.name}
 */
object App {
  
  def foo(x : Array[String]) = x.foldLeft("")((a,b) => a + b)

  // schema of tshark
  val schema = StructType(
    StructField("ip_src", StringType, nullable = true) ::
      StructField("ip_dst", StringType, nullable = true) ::
      StructField("ip_len", DoubleType, nullable = true) ::
      StructField("eth_src", StringType, nullable = true) ::
      StructField("eth_dst", StringType, nullable = true) ::
      StructField("tcp_src_port", IntegerType, nullable = true) ::
      StructField("tcp_dst_port", IntegerType, nullable = true) ::
      StructField("frame_time_epoch", StringType, nullable = true) ::
      StructField("frame_len", DoubleType, nullable = true) ::
      StructField("frame_protocols", StringType, nullable = true) ::
      StructField("prepared_frame_time", StringType, nullable = true) ::
      Nil
  )

  
  def main(args : Array[String]) {

    // context for spark
    val spark = SparkSession.builder
      .master("local[*]")
      .appName("lambda")
      .getOrCreate()

    // read tshark.csv file to data frame
    val df = spark.read.format("csv")
      .option("header", value = true)
      .option("delimiter", "|")
      .option("timestampFormat", "MMM dd yyyy HH:mm:ss.S")
      //.option("mode", "DROPMALFORMED")
      .schema(schema)
      .load("src/rsc/tshark.csv")
      .cache()
    df.printSchema()
    df.show(truncate = false)

    // prepared_frame_time includes a space at the end.
    // df.withColumn("length_of_frame_time", length(col("prepared_frame_time"))).show(false)

    val tsDf = df.withColumn("transform_frame_time", lpad(col("prepared_frame_time"), 30, "X"))
      .withColumn("frame_time", to_timestamp(col("transform_frame_time"), "MMM  d yyyy HH:mm:ss.SSSSSSSSS"))
      .drop(col("prepared_frame_time, transform_frame_time"))

    tsDf.show(true)

    // add unix timestamp to data frame
    val tDf = tsDf.withColumn("timestamp", unix_timestamp(col("frame_time")))
    tDf.show(truncate = false)

    // group with 1 minute time, ip_src, frame_protocols
    // calculate matrix(count, average len, local anomaly)
    val gDf = tDf
      .groupBy(
        window(col("frame_time"), "1 minutes")
          .alias("time_window"), col("ip_src"), col("frame_protocols")
    )
      .agg(
        count("*").as("count"),
        avg("ip_len").as("ip_avg_len"),
        avg("frame_len").as("frame_avg_len"),
        (avg("ip_len") / max("ip_len")).as("ip_local_anomaly"),
        (avg("frame_len") / max("frame_len")).as("frame_local_anomaly")
      )
      .withColumn("start", col("time_window")("start"))
      .withColumn("end", col("time_window")("end"))
    tDf.printSchema()
    gDf.printSchema()
    gDf.show(truncate = false)



    // join tDf and gDf
    // join with ip_src, frame_protocols, frame_time (in range of start and end)
    // select only wanted field
    //df.as("a").join(df.as("b"), $"a.id" > $"b.id")`
    //val cond = tDf("ip_src") === gDf("ip_src") && tDf("frame_protocols") === gDf("frame_protocols") && tDf("frame_time") >= gDf("start") && tDf("frame_time") <= gDf("end")

    val joindDF = tDf.as("a").join(gDf.as("b"),
                  col("a.ip_src") === col("b.ip_src") &&
                  col("a.frame_protocols") === col("b.frame_protocols") &&
                  col("a.frame_time") >= col("b.start") &&
                    col("a.frame_time") <= col("b.end")
    )
      .select(col("a.ip_src"),
              col("b.ip_src").as("b_ip_src")
              col()
      )
      .show(false)


//    val jDf = tDf.as("t").join(gDf.as("g"),
//              tDf("ip_src") === gDf("ip_src")
//              && tDf("frame_protocols") === gDf("frame_protocols")
//              && tDf("frame_time") >= gDf("start")
//              && tDf("frame_time") <= gDf("end")
//        , "left")
//        .select(tDf.col("ip_src").as("t_ip_dst"),
//          gDf.col("frame_protocols").as("g_frame_protocols")
//
//        )






    //      .select(tDf.col("ip_src"))
//              tDf("ip_dst").as("t_ip_dst"),
//              tDf("ip_len").as("t_ip_len"),
//              tDf("tcp_src_port").as("t_tcp_src_port"),
//              tDf("tcp_dst_port").as("t_tcp_dst_port"),
//              tDf("frame_protocols").as("t_frame_protocols"),
//              tDf("frame_len").as("t_frame_len"),
//              tDf("frame_time").as("t_frame_time"),
//              tDf("timestamp").as("t_timestamp"),
//              gDf("ip_avg_len").as("g_ip_avg_len"),
//              gDf("frame_avg_len").as("g_frame_avg_len"),
//              gDf("ip_local_anomaly").as("g_ip_local_anomaly"),
//              gDf("frame_local_anomaly").as("g_frame_local_anomaly"),
//              gDf("count").as("g_count"))
//    jDf.printSchema()
//    jDf.show(truncate = false)
//
//    // add feature column
//    val cols = Array("ip_avg_len", "frame_avg_len", "ip_local_anomaly", "frame_local_anomaly", "count")
//    val ass = new VectorAssembler().setInputCols(cols).setOutputCol("features")
//    val fDf = ass.transform(jDf)
//    fDf.printSchema()
//    fDf.show(truncate = false)
//
//    // split data set training(70%) and test(30%)
//    val seed = 5043
//    val Array(trnDta, tstDta) = fDf.randomSplit(Array(0.7, 0.3), seed)
//
//    // isolation forest model
//    val isf = new IsolationForest()
//      .setNumEstimators(100)
//      .setBootstrap(false)
//      .setMaxSamples(256)
//      .setMaxFeatures(5)
//      .setFeaturesCol("features")
//      .setPredictionCol("predicted_label")
//      .setScoreCol("outlier_score")
//      .setContamination(0.1)
//      .setRandomSeed(1)
//    val model = isf.fit(trnDta)
//
//    // test the model with test data set
//    val pDf = model.transform(tstDta)
//    pDf.printSchema()
//    pDf.show(truncate = false)

    // data frame with with required fields(e.g for visualization)
    //val vDf = pDf.select($"timestamp", $"features", $"predicted_label", $"outlier_score")
      //.withColumn("id", monotonically_increasing_id())
    //vDf.show(truncate = false)

    // create temp table for sql visualization
    //vDf.createOrReplaceTempView("iforest")

  }

}
