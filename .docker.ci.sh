command="docker-compose -f docker-compose.ci.yml up"
log="ci-docker-compose.log"

$command 2>&1 | tee "$log" &
pid=$!

while sleep 10
do
    if fgrep --quiet "[33mmain_1  |[0m Rocket has launched from http://0.0.0.0:80" "$log"
    then
        kill $pid
        exit 0
    fi
done

# Travis will kill if inactive for 10m