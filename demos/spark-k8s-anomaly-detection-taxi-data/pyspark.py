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
#df = spark.read.parquet("/home/andrew/Downloads/fhvhv_tripdata_2020-09.parquet")

url = "https://repo.stackable.tech/repository/misc/ny-taxi-data/fhvhv_tripdata_2020-09.parquet"
spark.sparkContext.addFile(url)
df = spark.read.parquet("file://"+SparkFiles.get("fhvhv_tripdata_2020-09.parquet"))

dfs = df.select(
    to_date(df.pickup_datetime).alias("day_date")
    , hour(df.pickup_datetime).alias("hour")
    , minute(df.pickup_datetime).alias("minute")
    , df["driver_pay"]
)

dfs = dfs.withColumn("minute_group", when(dfs.minute < 30, '00').otherwise('30'))
dfs = dfs.withColumn("time_group",concat_ws(":", lpad(dfs.hour, 2, '0'), dfs.minute_group, lit('00')))
dfs = dfs.withColumn("ts",concat_ws(" ", dfs.day_date, dfs.time_group))

dfs2 = dfs.select(
    date_format(dfs.ts, "yyyy-MM-dd HH:mm:ss").alias("date_group")
    , year(dfs.ts).alias('year')
    , month(dfs.ts).alias('month')
    , dayofmonth(dfs.ts).alias("dayofmonth")
    , dayofweek(dfs.ts).alias("dayofweek")
    , dfs.hour
    , dfs.driver_pay
).groupby("date_group", "hour", "year", "month", "dayofmonth", "dayofweek").agg(functions.count('driver_pay').alias('no_rides'), functions.round(functions.sum('driver_pay'), 2).alias('total_bill'), functions.round(functions.avg('driver_pay'), 2).alias('avg_bill')).orderBy("date_group")

dfs2.show()

windowSpec  = Window.partitionBy("hour").orderBy("date_group")

dfs2 = dfs2.withColumn("lag",lag("no_rides",2).over(windowSpec))
dfs2 = dfs2.filter("lag IS NOT NULL")
dfs2.show()

# instantiate a scaler, an isolation forest classifier and convert the data into the appropriate form
scaler = StandardScaler()
classifier = IsolationForest(contamination=0.3, random_state=42, n_jobs=-1)

dfs2 = dfs2.select(dfs2.hour, dfs2.year, dfs2.month, dfs2.dayofmonth, dfs2.dayofweek, dfs2.no_rides, dfs2.total_bill, dfs2.avg_bill, dfs2.lag)

x_train = scaler.fit_transform(dfs2.collect())
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

dfs2 = dfs2.withColumn(
    'prediction',
    udf_predict_using_broadcasts('hour', 'year', 'month', 'dayofmonth', 'dayofweek', 'no_rides', 'total_bill', 'avg_bill', 'lag')
)
dfs2.show()

######################
data = [
    {'feature1': 1., 'feature2': 0., 'feature3': 0.3, 'feature4': 0.01},
    {'feature1': 10., 'feature2': 3., 'feature3': 0.9, 'feature4': 0.1},
    {'feature1': 101., 'feature2': 13., 'feature3': 0.9, 'feature4': 0.91},
    {'feature1': 111., 'feature2': 11., 'feature3': 1.2, 'feature4': 1.91},
]

from pyspark.sql import functions, types
from sklearn.ensemble import IsolationForest
from sklearn.preprocessing import StandardScaler

df = spark.createDataFrame(data)

scaler = StandardScaler()
classifier = IsolationForest(contamination=0.3, random_state=42, n_jobs=-1)

x_train = scaler.fit_transform(x_train)
clf = classifier.fit(x_train)

SCL = spark_session.sparkContext.broadcast(scaler)
CLF = spark_session.sparkContext.broadcast(clf)

def predict_using_broadcasts(feature1, feature2, feature3, feature4):
    prediction = 0
    x_test = [[feature1, feature2, feature3, feature4]]
    try:
        x_test = SCL.value.transform(x_test)
        prediction = CLF.value.predict(x_test)[0]
    except ValueError:
        import traceback
        traceback.print_exc()
        print('Cannot predict:', x_test)
    return int(prediction)


udf_predict_using_broadcasts = functions.udf(predict_using_broadcasts, types.IntegerType())

df = df.withColumn(
    'prediction',
    udf_predict_using_broadcasts('feature1', 'feature2', 'feature3', 'feature4')
)
