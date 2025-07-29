#!/bin/bash

export DOCKER_BUILDKIT=1

docker buildx build \
    --platform linux/amd64,linux/arm64/v8 \
    --cache-from type=registry,ref=shymike/rustytime:cache \
    --cache-to type=registry,ref=shymike/rustytime:cache,mode=max \
    --tag shymike/rustytime:latest \
    --push \
    .