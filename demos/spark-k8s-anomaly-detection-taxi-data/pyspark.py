"""
anomaly detection
-----------------
read data in spark:
"""

from pyspark import SparkFiles
from pyspark.sql.functions import dayofweek, to_date, date_format, year, hour, minute, month, when, dayofmonth, dayofweek
from pyspark.sql.functions import concat_ws, substring, concat, lpad, lit
from pyspark.sql.functions import round, sum, count, avg
from pyspark.sql.functions import lag
from pyspark.sql.window import Window
from pyspark.sql import functions, types
from sklearn.ensemble import IsolationForest
from sklearn.preprocessing import StandardScaler

spark = SparkSession.builder.appName("NY TLC AD").getOrCreate()

url = "https://repo.stackable.tech/repository/misc/ny-taxi-data/fhvhv_tripdata_2020-09.parquet"
spark.sparkContext.addFile(url)
input_df = spark.read.parquet("file://"+SparkFiles.get("fhvhv_tripdata_2020-09.parquet"))

df = input_df.select(
    to_date(input_df.pickup_datetime).alias("day_date")
    , year(input_df.pickup_datetime).alias('year')
    , month(input_df.pickup_datetime).alias('month')
    , dayofmonth(input_df.pickup_datetime).alias("dayofmonth")
    , dayofweek(input_df.pickup_datetime).alias("dayofweek")
    , hour(input_df.pickup_datetime).alias("hour")
    , minute(input_df.pickup_datetime).alias("minute")
    , input_df.driver_pay
)

df = df.withColumn("minute_group", when(df.minute < 30, '00').otherwise('30'))
df = df.withColumn("time_group",concat_ws(":", lpad(df.hour, 2, '0'), df.minute_group, lit('00')))
df = df.withColumn("ts",concat_ws(" ", df.day_date, df.time_group))

dfs = df.select(
    date_format(df.ts, "yyyy-MM-dd HH:mm:ss").alias("date_group")
    , df.year
    , df.hour
    , df.month
    , df.dayofmonth
    , df.dayofweek
    , df.driver_pay
).groupby("date_group", "hour", "year", "month", "dayofmonth", "dayofweek").agg(functions.count('driver_pay').alias('no_rides'), functions.round(functions.sum('driver_pay'), 2).alias('total_bill'), functions.round(functions.avg('driver_pay'), 2).alias('avg_bill')).orderBy("date_group")

dfs.show()

windowSpec  = Window.partitionBy("hour").orderBy("date_group")

dfs = dfs.withColumn("lag",lag("no_rides",2).over(windowSpec))
dfs = dfs.filter("lag IS NOT NULL")
dfs.show()

scaler = StandardScaler()
classifier = IsolationForest(contamination=0.005, n_estimators=200, max_samples=0.7, random_state=42, n_jobs=-1)

df_model = dfs.select(dfs.hour, dfs.year, dfs.month, dfs.dayofmonth, dfs.dayofweek, dfs.no_rides, dfs.total_bill, dfs.avg_bill, dfs.lag)

x_train = scaler.fit_transform(df_model.collect())
clf = classifier.fit(x_train)

SCL = spark.sparkContext.broadcast(scaler)
CLF = spark.sparkContext.broadcast(clf)

def predict_using_broadcasts(hour, year, month, dayofmonth, dayofweek, no_rides, total_bill, avg_bill, lag):
    prediction = 0
    x_test = [[hour, year, month, dayofmonth, dayofweek, no_rides, total_bill, avg_bill, lag]]
    try:
        x_test = SCL.value.transform(x_test)
        prediction = CLF.value.predict(x_test)[0]
    except ValueError:
        import traceback
        traceback.print_exc()
        print('Cannot predict:', x_test)
    return int(prediction)

udf_predict_using_broadcasts = functions.udf(predict_using_broadcasts, types.IntegerType())

df_pred = df_model.withColumn(
    'prediction',
    udf_predict_using_broadcasts('hour', 'year', 'month', 'dayofmonth', 'dayofweek', 'no_rides', 'total_bill', 'avg_bill', 'lag')
)
df_pred.show()
