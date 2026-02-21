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

```
py setup.py
```

## Screenshots

![Dashboard](dashboard.png)
![Server Details](server.png)

## License

This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.
