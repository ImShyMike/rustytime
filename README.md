# rustytime

🕒 blazingly fast time tracking for developers

---

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
curl -O https://raw.githubusercontent.com/ImShyMike/rustytime/main/rustytime/.env.example
mv .env.example .env
# Edit the .env file before running docker compose
docker compose -f https://raw.githubusercontent.com/ImShyMike/rustytime/main/docker-compose.yml up
```
