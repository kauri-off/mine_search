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

## masscan_wrapper

> **Linux only** — `masscan` requires raw socket access and is only supported on Linux.

`masscan_wrapper` is a CLI utility (written in Rust) that reads a `masscan` grepable output file and bulk-imports discovered IPs into the mine_search API.

### Prerequisites

- Linux
- [Rust / Cargo](https://rustup.rs/) installed
- [`masscan`](https://github.com/robertdavidgraham/masscan) installed (`sudo apt install masscan` or equivalent)

### Build

```bash
cd masscan_wrapper
cargo build --release
# Binary will be at: ./target/release/masscan_wrapper
```

### Usage

#### Step 1 — Scan for Minecraft servers with masscan

Run `masscan` against your target IP range and save the results in grepable (`-oG`) format:

```bash
sudo masscan <range> -p25565 --rate 10000 -oG output.txt
```

Replace `<range>` with the IP range you want to scan, e.g. `10.0.0.0/8`.

#### Step 2 — Import results into the API

```bash
./target/release/masscan_wrapper \
  --endpoint https://example.com \
  --password <your-password> \
  --file output.txt
```

| Flag | Short | Description |
|------|-------|-------------|
| `--endpoint` | `-e` | Base URL of the mine_search API (e.g. `https://example.com`) |
| `--password` | `-p` | API login password |
| `--file` | `-f` | Path to the masscan `-oG` output file |

The tool will:
1. Log in to the API using the provided password.
2. Parse the masscan output file and extract all discovered IPs.
3. Show you how many IPs were found and prompt for confirmation before importing.
4. POST the IP list to the API.

#### Example session

```
[*] Logging in to https://example.com...
[+] Logged in successfully.
[*] Reading masscan output from output.txt...
[+] Found 42 IPs.

Add these 42 IP(s) to https://example.com? [y/N] y
[*] Done.
```

### Notes

- The wrapper expects masscan output in **grepable format** (`-oG`). Other output formats (JSON, XML, binary) are not supported.
- Masscan must be run as **root** (or with `sudo`) because it requires raw socket access.
- Rate (`--rate`) controls packets per second — adjust based on your network and target. High rates may trigger network alarms or cause packet loss.


## License

This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.
