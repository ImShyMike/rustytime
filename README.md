# [rustytime](https://rustytime.shymike.dev)

🕒 blazingly fast time tracking for developers

---

> If you want to try this project out, you can head over to [https://rustytime.shymike.dev](https://rustytime.shymike.dev) for the deployed version!

## Features

- ✅ Time tracking
- ✅ Stat visualization
- ✅ WakaTime compatible
- ✅ Fast and memory efficient

## What is this?

`rustytime` is a [WakaTime](https://wakatime.com) compatible server that can be used to track time in any program with any of the existing [plugins](https://wakatime.com/plugins)!

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

## Usage

Run the docker compose and open the website at [http://localhost:3000](http://localhost:3000)

### Seeding the DB

The `seed` feature can be enabled in the build that seeds the database with a single user and 10000 heartbeats.

```bash
cargo run --features seed
```

### Using cloudflare

To get the actual client IP's when behind cloudflare, enable the `cloudflare` feature.

```bash
cargo run --features cloudflare
```

## WakaTime

When using a WakaTime client, point your requests to `http://localhost:3000/api/v1` (or `https://api-rustytime.shymike.dev/api/v1` if using the deployed version)

## License

This project is licensed under the [GNU AGPLv3](https://github.com/ImShyMike/rustytime/blob/HEAD/LICENSE)
