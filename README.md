# MineSearch Setup Guide

**MineSearch** is a powerful search engine for Minecraft servers. It scans and indexes active servers and nicknames so you can easily find a server and crash it.

> [!WARNING]
> RUNNING THE WORKER MAY RESULT IN YOUR SERVER BEING BLOCKED DUE TO ABUSIVE SCANNING ACTIVITIES. MAKE SURE YOU UNDERSTAND THE RISKS INVOLVED.

> [!IMPORTANT]
> If you want to run the worker on a different server, please refer to the separate documentation here: [Worker Setup Guide](separate_deploy.md).

## Environment Variables

Create a `.env` file in the root directory and add the following values:

```env
POSTGRES_USER=user
POSTGRES_PASSWORD=password
POSTGRES_DB=mine_search_db

BACKEND_PASSWORD=password

THREADS=900
ONLY_UPDATE=false
```

## Installation

### Prerequisites

- [Docker](https://www.docker.com/) installed
- [Docker Compose](https://docs.docker.com/compose/) installed

### 1. Clone the repository

```bash
git clone https://github.com/kauri-off/mine_search
cd mine_search
```

### 2. Install Diesel CLI

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/diesel-rs/diesel/releases/latest/download/diesel_cli-installer.sh | sh
```

To verify the installation, run:

```bash
diesel --version
```

### 3. Start PostgreSQL

```bash
docker compose up postgres -d
```

### 4. Set up environment variables

```bash
export $(grep -v '^#' .env | xargs)
export DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@127.0.0.1:5432/${POSTGRES_DB}"
```

### 5. Run database migrations

```bash
cd db_schema
~/.cargo/bin/diesel migration run
```

## Running the Application

You can run the application in two ways: by building the code locally or by using prebuilt images from GitHub Container Registry (GHCR).

### **Option 1: Build Locally**

You don't have to modify anything; Docker Compose is already configured to build locally.

Run the application:

```bash
docker compose up -d --build
```

### **Option 2: Use Prebuilt Images**

Modify the `docker-compose.yml` file to use GHCR images:

```yaml
services:
  worker:
    #    build:
    #      context: .
    #      dockerfile: worker.Dockerfile
    image: ghcr.io/kauri-off/mine_search/worker:latest

  backend:
    #    build:
    #      context: .
    #      dockerfile: backend.Dockerfile
    image: ghcr.io/kauri-off/mine_search/backend:latest
```

Run the application:

```bash
docker compose up -d
```

## Configure a Domain and Proxy

If you have a domain and want to configure it for reverse proxy, run:

```bash
./setup.sh
```

This will update the domain name in Nginx and Caddy configuration files.

## License

This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.

