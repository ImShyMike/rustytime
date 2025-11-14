#!/bin/bash

export DOCKER_BUILDKIT=1

docker buildx build \
    --platform linux/amd64,linux/arm64/v8 \
    --cache-from type=local,src=./.buildx-cache \
    --cache-to type=local,dest=./.buildx-cache,mode=max \
    --tag shymike/rustytime:latest \
    --push \
    .