#!/bin/bash

set -ex

cmd="$1"
version="$2"

if [ -z "$cmd" ]; then
    echo "missing command..."
    exit 1
elif [ "$cmd" = "build" ]; then
    docker build -t jaemk/upaste:latest .
    if [ ! -z "$version" ]; then
        docker build -t jaemk/upaste:$version .
    fi
elif [ "$cmd" = "run" ]; then
    # hint, volume required: docker volume create upastedata
    docker run --rm --init -p 3900:3900 --env-file .env.docker --mount source=upastedata,destination=/upaste/db jaemk/upaste:latest
elif [ "$cmd" = "shell" ]; then
    docker run --rm --init -p 3900:3900 --env-file .env.docker --mount source=upastedata,destination=/upaste/db jaemk/upaste:latest /bin/bash
elif [ "$cmd" = "migrate" ]; then
    docker run --rm --init -p 3900:3900 --env-file .env.docker --mount source=upastedata,destination=/upaste/db jaemk/upaste:latest ./bin/upaste admin database migrate
fi
