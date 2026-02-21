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

# Installation

## Prerequisites

Ensure you have **Python 3** installed. Download it from [python.org](https://www.python.org/downloads/).

## 1. Create a Virtual Environment

Create a `.venv` in the project root:

```bash
python -m venv .venv
```

## 2. Activate the Virtual Environment

**Windows:**

```bash
.venv\Scripts\activate
```

**macOS / Linux:**

```bash
source .venv/bin/activate
```

## 3. Install Dependencies

Install the required packages:

```bash
pip install -r requirements.txt
```

## 4. Run Setup

```bash
python setup.py
```

> 💡 To deactivate the virtual environment when you're done, simply run `deactivate`.

## Screenshots

![Dashboard](dashboard.png)
![Server Details](server.png)

## License

This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.
