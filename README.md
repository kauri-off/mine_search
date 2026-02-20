# <img src="frontend/public/favicon.png" width="32" height="32"> MineSearch

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

**MineSearch** is a powerful search engine for Minecraft servers. It scans and indexes active servers and nicknames so you can easily find a server and analyze its data.

> [!CAUTION]
> RUNNING THE WORKER MAY RESULT IN YOUR SERVER BEING BLOCKED DUE TO ABUSIVE SCANNING ACTIVITIES. MAKE SURE YOU UNDERSTAND THE RISKS INVOLVED.

## Tech Stack

| Layer    | Technology                                                         |
| -------- | ------------------------------------------------------------------ |
| Frontend | React 19, TypeScript, Vite, Tailwind CSS, TanStack Query, Recharts |
| Backend  | Rust, Axum, Diesel, PostgreSQL                                     |
| Worker   | Rust, Tokio, Diesel, PostgreSQL                                    |

## Installation

Choose the installation guide for your operating system:

- [Linux Installation Guide](LINUX_INSTALL.md)
- [Windows Installation Guide](WINDOWS_INSTALL.md)

After completing the installation steps, come back here to run the application.

## Running the Application

You can run the application in two ways: by building the code locally or by using prebuilt images from GitHub Container Registry (GHCR).

### **Option 1: Use Prebuilt Images (Default)**

The `docker-compose.yml` file is configured by default to use prebuilt images from GitHub Container Registry (GHCR) for the `worker` and `backend` services. No modifications are needed.

To verify, check that the `worker` and `backend` services in `docker-compose.yml` have the `image` lines uncommented and the `build` sections commented out (this is the default state):

```yaml
services:
  worker:
    #build:
    #  context: .
    #  dockerfile: worker.Dockerfile
    image: ghcr.io/kauri-off/mine_search/worker:latest

  backend:
    #build:
    #  context: .
    #  dockerfile: backend.Dockerfile
    image: ghcr.io/kauri-off/mine_search/backend:latest
```

Run the application:

```bash
docker compose up -d
```

### **Option 2: Build Locally**

To build all services locally, modify the `docker-compose.yml` file by uncommenting the `build` sections and commenting out the `image` lines for the `worker` and `backend` services:

```yaml
services:
  worker:
    build:
      context: .
      dockerfile: worker.Dockerfile
    # image: ghcr.io/kauri-off/mine_search/worker:latest

  backend:
    build:
      context: .
      dockerfile: backend.Dockerfile
    # image: ghcr.io/kauri-off/mine_search/backend:latest
```

> **Note:** The `frontend` service is always built locally regardless of which option you choose.

Run the application:

```bash
docker compose up -d --build
```

## Screenshots

![Dashboard](dashboard.png)
![Server Details](server.png)

## License

This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.
