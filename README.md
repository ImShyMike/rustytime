# rustytime

🕒 blazingly fast time tracking for developers

---

## Features

- ✅ Time tracking
- ✅ Stat visualization
- ✅ WakaTime compatible
- ✅ Fast and memory efficient

## What is this?

`rustytime` is a simple WakaTime compatible backend that can be used to track time in any program with an existing [plugin](https://wakatime.com/plugins)!

## Installation

### Compiling from source

Dependencies:

- rust
- docker
- git

```bash
git clone https://github.com/ImShyMike/rustytime.git
cd rustytime
mv .env.example .env
# Edit the .env file before running docker compose
docker compose up
```

### Pulling containers from dockerhub

Dependencies:

- docker
- curl

```bash
mkdir rustytime
cd rustytime
curl -O https://raw.githubusercontent.com/ImShyMike/rustytime/main/.env.example
curl -o docker-compose.yml https://raw.githubusercontent.com/ImShyMike/rustytime/main/docker-compose.yml
mv .env.example .env
# Edit the .env file before running docker compose
docker compose up
```
