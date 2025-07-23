#!/bin/bash

docker buildx build \
  --platform linux/amd64 \
  --cache-from shymike/rustytime:cache \
  --cache-to shymike/rustytime:cache \
  --tag shymike/rustytime:latest \
  --push .