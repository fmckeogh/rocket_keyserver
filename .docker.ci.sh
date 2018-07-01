command="docker-compose -f docker-compose.ci.yml up"
log="ci-docker-compose.log"

$command > "$log" 2>&1 &
pid=$!

while sleep 10
do
    if fgrep --quiet "Rocket has launched from http://0.0.0.0:80" "$log"
    then
        kill $pid
        exit 0
    fi
done

# Travis will kill if inactive for 10m