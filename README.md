<div align="center">
    <a href="https://rustytime.shymike.dev">
        <img src="frontend/static/pwa/favicon-196.png" alt="rustytime" width="200">
    </a>
</div>
<div id="user-content-toc" align="center">
  <ul align="center" style="list-style: none;">
    <summary >
      <h1><a href="https://rustytime.shymike.dev">rustytime</a></h1>
    </summary>
  </ul>
</div>
<p align="center">🕒 blazingly fast time tracking for developers</p>

<div align="center">
    <a href="https://crates.io/crates/rustytime-server"><img alt="Crates.io Total Downloads" src="https://img.shields.io/crates/d/rustytime-server?style=flat-square&color=blue"></a>
    <a href="https://github.com/ImShyMike/rustytime/actions/workflows/ci.yml"><img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/ImShyMike/rustytime/ci.yml?style=flat-square&color=green"></a>
    <a href="https://crates.io/crates/rustytime-server"><img alt="GitHub last commit" src="https://img.shields.io/github/last-commit/ImShyMike/rustytime?style=flat-square&color=yellowgreen"></a>
    <a href="https://crates.io/crates/rustytime-server"><img alt="Crates.io Version" src="https://img.shields.io/crates/v/rustytime-server?style=flat-square&color=yellow"></a>
    <a href="https://crates.io/crates/rustytime-server"><img alt="Crates.io License" src="https://img.shields.io/crates/l/rustytime-server?style=flat-square&color=orange"></a>
    <a href="https://crates.io/crates/rustytime-server"><img alt="Crates.io Size" src="https://img.shields.io/crates/size/rustytime-server?style=flat-square&color=red"></a>
</div>

---

## Features

- ✅ Time tracking
- ✅ Stat visualization
- ✅ WakaTime compatible
- ✅ Fast and memory efficient

## What is this?

`rustytime` is a [WakaTime](https://wakatime.com) compatible server that can be used to track time in most apps with any of the existing [plugins](https://wakatime.com/plugins)!

## Local Development

```sh
# Clone the repo
$ git clone https://github.com/ImShyMike/rustytime && cd rustytime

# Copy the env file
$ cp .env.example .env
```

Edit your `.env` file to include the following:

```env
# GitHub OAuth Settings
GITHUB_CLIENT_ID=client_id_goes_here
GITHUB_CLIENT_SECRET=client_secret_goes_here
```

### Build & Run

```sh
# Run the full app
$ docker compose up

# OR

# Run the databse + backend 
$ docker compose up timescaledb rustytime
# Run the frontend
$ cd frontend && bun run dev
```

The app should now be available at [http://localhost:5173](http://localhost:5173)

### Observability (OTel + LGTM)

If you're running the self-hosted Grafana LGTM (Loki/Grafana/Tempo/Mimir) stack or an OpenTelemetry Collector on the same machine, expose its OTLP receiver (default gRPC on `4317`). Then add the following to your `.env` (already scaffolded in `.env.example`):

```env
OTEL_SERVICE_NAME=rustytime-server
OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://127.0.0.1:4317
OTEL_EXPORTER_OTLP_TRACES_PROTOCOL=grpc
OTEL_RESOURCE_ATTRIBUTES=deployment.environment=development
# Only when your collector requires authentication headers (comma-separated k=v)
# OTEL_EXPORTER_OTLP_HEADERS=x-otlp-token=changeme
```

Restart the backend (`cargo run`) after updating the env vars. The warning about missing OTEL exporter configuration disappears, and spans begin flowing into Tempo. From there you can wire Grafana dashboards against the same Tempo instance alongside your LGTM stack.

### Seeding the DB

The `seed` feature can be enabled in the build to seed the database with a single user and 10000 heartbeats.

```bash
cargo run --features seed
```

## WakaTime

When using a WakaTime client, point your requests to `http://localhost:3000/api/v1` (or `https://api-rustytime.shymike.dev/api/v1` if using the deployed version)

## Star History

<a href="https://www.star-history.com/#imshymike/rustytime&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=imshymike/rustytime&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=imshymike/rustytime&type=date&legend=top-left" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=imshymike/rustytime&type=date&legend=top-left" />
 </picture>
</a>

## License

This project is licensed under the [GNU AGPLv3](https://github.com/ImShyMike/rustytime/blob/HEAD/LICENSE)
