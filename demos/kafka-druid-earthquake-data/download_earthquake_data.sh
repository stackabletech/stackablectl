# This scripts is not used for the demo
# Its purpose is to document how to retrive the used earthquake data

# rm -f 20* earthquakes.csv

for year in {2000..2022}; do
    for month in {01..12}; do
        start="${year}-${month}-01 00:00:00"
        end="${year}-${month}-31 23:59:59"
        echo "Downloading $year/$month"
        if ! test -f "${year}_${month}.csv"; then
            curl -o "${year}_${month}.csv" "https://earthquake.usgs.gov/fdsnws/event/1/query.csv?starttime=${start}&endtime=${end}&minmagnitude=1&orderby=time-asc"
        fi
        if test -f earthquakes.csv; then
            tail -n +2 "${year}_${month}.csv" >> earthquakes.csv
        else
            cp "${year}_${month}.csv" earthquakes.csv # Copy to preserve header
        fi
    done
done

# To filter duplicate lines
# cat earthquakes.csv | sort | uniq > earthquakes_uniq.csv
# To check no duplicate lines
# cat earthquakes_uniq.csv | sort | uniq -c | sort -r | head
