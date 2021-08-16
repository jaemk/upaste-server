#!/bin/bash

set -ex

cmd="$1"
version="$(git rev-parse HEAD | awk '{ printf "%s", substr($0, 0, 7) }')"

# options
reg="${REGISTRY:-docker.jaemk.me}"
port_map="${PORT_MAP:-127.0.0.1:3900:3003}"

if [ -z "$cmd" ]; then
    echo "missing command..."
    exit 1
elif [ "$cmd" = "build" ]; then
    if [ ! -z "$version" ]; then
        docker build -t $reg/upaste:$version .
    fi
    docker build -t $reg/upaste:latest .
elif [ "$cmd" = "push" ]; then
    $0 build
    docker push $reg/upaste:$version
    docker push $reg/upaste:latest
elif [ "$cmd" = "run" ]; then
    # hint, volume required: docker volume create upastedata
    $0 build
    docker run --rm -it --init -p 3900:3003 --env-file .env.docker --mount source=upastedata,destination=/upaste/db $reg/upaste:latest
elif [ "$cmd" = "shell" ]; then
    $0 build
    docker run --rm -it --init -p 3900:3003 --env-file .env.docker --mount source=upastedata,destination=/upaste/db $reg/upaste:latest /bin/bash
elif [ "$cmd" = "migrate" ]; then
    $0 build
    docker run --rm -it --init -p 3900:3003 --env-file .env.docker --mount source=upastedata,destination=/upaste/db $reg/upaste:latest ./bin/upaste admin database migrate
fi
