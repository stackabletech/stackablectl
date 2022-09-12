# This script is not used for the demo
# Its purpose is to document how to retrieve the used earthquake data

# To delete the csv files downloaded
# rm -f 19*.csv 20*.csv earthquakes.csv

# We have to request small chunks otherwise the API will not return us any data
for year in {1950..1999}; do
    for month in {01..12}; do
        # Either day 01 - 15
        # or day     16 - 31
        for startDay in 01 16; do
            if [ "$startDay" == "01" ]; then
                endDay="15"
            else
                endDay="31"
            fi
            start="${year}-${month}-${startDay} 00:00:00"
            end="${year}-${month}-${endDay} 23:59:59"
            echo "Downloading $year/$month/${startDay} (from $start to $end)"
            if ! test -f "${year}_${month}_${startDay}.csv"; then
                curl -o "${year}_${month}_${startDay}.csv" "https://earthquake.usgs.gov/fdsnws/event/1/query.csv?starttime=${start}&endtime=${end}&minmagnitude=1&orderby=time-asc"
            fi
            if test -f earthquakes.csv; then
                tail -n +2 "${year}_${month}_${startDay}.csv" >> earthquakes.csv
            else
                cp "${year}_${month}_${startDay}.csv" earthquakes.csv # Copy to preserve header
            fi
        done
    done
done

# To filter duplicate lines
# cat earthquakes.csv | sort | uniq > earthquakes_uniq.csv
# To check no duplicate lines
# cat earthquakes_uniq.csv | sort | uniq -c | sort -r | head
