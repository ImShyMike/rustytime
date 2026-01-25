<div align="center">
  <a href="https://rustytime.shymike.dev">
    <img alt="rustytime" width="180" src="https://raw.githubusercontent.com/ImShyMike/rustytime/refs/heads/main/frontend/static/pwa/favicon-196.png">
  </a>
  <div id="user-content-toc" align="center">
    <ul>
      <summary><h1>rustytime</h1></summary>
      <summary><p><strong>🕒 Blazingly fast time tracking for developers</strong></p></summary>
    </ul>
  </div>

  <a href="https://crates.io/crates/rustytime-server"><img alt="Crates.io Downloads" src="https://img.shields.io/crates/d/rustytime-server?style=flat-square&color=blue"></a>
  <a href="https://github.com/ImShyMike/rustytime/actions/workflows/ci.yml"><img alt="CI Status" src="https://img.shields.io/github/actions/workflow/status/ImShyMike/rustytime/ci.yml?style=flat-square&color=green"></a>
  <a href="https://crates.io/crates/rustytime-server"><img alt="Version" src="https://img.shields.io/crates/v/rustytime-server?style=flat-square&color=yellow"></a>
  <a href="https://github.com/ImShyMike/rustytime/blob/HEAD/LICENSE"><img alt="License" src="https://img.shields.io/crates/l/rustytime-server?style=flat-square&color=orange"></a>
</div>

---

## What is rustytime?

`rustytime` is a self-hosted, [WakaTime](https://wakatime.com)-compatible backend
 for tracking your coding time that works with any* existing WakaTime [plugin](https://wakatime.com/plugins)!

*every plugin that allows setting custom API URLs

### Features

- **WakaTime Compatible** — Drop-in replacement for the WakaTime API
- **Fast and Memory Efficient** — Built for high performance and low resource usage
- **GitHub OAuth** — Simple login with your GitHub account
- **Data Importing** — Import your existing data from [Hackatime](https://hackatime.hackclub.com)
- **Observability** — Built-in OpenTelemetry + Pyroscope support
- **Self-Hosted** — Full control over your data

## Quick Start

### Prerequisites

- [Docker](https://docs.docker.com/get-docker) (and Docker Compose)
- [GitHub OAuth App](https://github.com/settings/developers) (for authentication)

### 1. Clone and Configure

```bash
git clone https://github.com/ImShyMike/rustytime && cd rustytime
cp .env.example .env
```

Edit the `.env` file with your GitHub OAuth credentials:

```env
GITHUB_CLIENT_ID=your_client_id
GITHUB_CLIENT_SECRET=your_client_secret
```

### 2. Run with Docker

```bash
docker compose up
```

The app should now be now available at **[http://localhost:5173](http://localhost:5173)**

### Alternative: Run Components Separately

```bash
# Start database + backend only
docker compose up timescaledb rustytime

# Run frontend (in another terminal)
cd frontend && npm run dev
```

## Connect Your Editor

Configure your WakaTime plugin to use rustytime:

| Environment | API URL |
| ------------- | --------- |
| **Local** | `http://localhost:3000/api/v1` |
| **Hosted** | `https://api-rustytime.shymike.dev/api/v1` |

Most plugins support setting a custom API URL in their settings.
Use your rustytime API key from the settings page.

## Development

### Backend (Rust)

```bash
cd rustytime
cargo run                    # Start server
cargo run --features seed    # Start with test data (10k heartbeats)
cargo test                   # Run tests
cargo build --release        # Production build
```

### Frontend (SvelteKit)

```bash
cd frontend
npm run dev      # Start dev server
npm run build    # Production build
npm run check    # Type check
npm run lint     # Lint code
```

## Observability

### OpenTelemetry

Enable tracing, metrics, and logs with OpenTelemetry:

```env
OTEL_SERVICE_NAME=rustytime-backend
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_EXPORTER_OTLP_PROTOCOL=grpc
OTEL_TRACES_EXPORTER=otlp
OTEL_METRICS_EXPORTER=otlp
OTEL_LOGS_EXPORTER=otlp
```

### Pyroscope Profiling

Enable continuous profiling:

```env
PYROSCOPE_SERVER_URL=http://localhost:4040
PYROSCOPE_SAMPLE_RATE=99
```

## Is this better than wakapi?

Probably not... this was just a fun side project, if you want something that's
actually production ready just use [Wakapi](https://wakapi.dev) 😭

## Architecture

| Component | Details |
| --------- | ------------ |
| **Backend** | [Rust](https://rust-lang.org), [Axum](https://github.com/tokio-rs/axum), [Diesel](https://diesel.rs) |
| **Frontend** | [SvelteKit](https://github.com/sveltejs/kit), [TailwindCSS](https://tailwindcss.com) |
| **Database** | [TimescaleDB](https://github.com/timescale/timescaledb) ([PostgreSQL](https://www.postgresql.org)) |

## Star History

<a href="https://www.star-history.com/#imshymike/rustytime&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=imshymike/rustytime&type=date&theme=dark&legend=top-left"/>
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=imshymike/rustytime&type=date&legend=top-left"/>
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=imshymike/rustytime&type=date&legend=top-left"/>
 </picture>
</a>

## License

This project is licensed under the [GNU AGPLv3](https://github.com/ImShyMike/rustytime/blob/HEAD/LICENSE).
