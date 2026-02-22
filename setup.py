#!/usr/bin/env python3
"""
mine_search — Setup Script
Modes: Install / Update / Change Settings
Platform independent (Windows, Linux, macOS)
"""

import os
import sys
import time
import shutil
import subprocess
from pathlib import Path
import yaml


# ── ANSI colours ──────────────────────────────────────────────────────────────
RESET  = "\033[0m"
BOLD   = "\033[1m"
CYAN   = "\033[96m"
GREEN  = "\033[92m"
YELLOW = "\033[93m"
RED    = "\033[91m"

def c(text, col):  return f"{col}{text}{RESET}"
def header(text):  print(f"\n{BOLD}{CYAN}{'─'*52}\n  {text}\n{'─'*52}{RESET}")
def success(text): print(c(f"  ✔  {text}", GREEN))
def warn(text):    print(c(f"  ⚠  {text}", YELLOW))
def error(text):   print(c(f"  ✖  {text}", RED))

# ── UI helpers ────────────────────────────────────────────────────────────────

def ask(prompt, default=""):
    suffix = f" [{default}]" if default else ""
    val = input(f"  {prompt}{suffix}: ").strip()
    return val if val else default

def ask_yes(prompt, default=True):
    suffix = "Y/n" if default else "y/N"
    val = input(f"  {prompt} [{suffix}]: ").strip().lower()
    if val == "":
        return default
    return val in ("y", "yes")

def menu(title, options):
    print(f"\n  {BOLD}{title}{RESET}")
    for i, opt in enumerate(options, 1):
        print(f"    {i}) {opt}")
    while True:
        raw = input("  Choose: ").strip()
        if raw.isdigit() and 1 <= int(raw) <= len(options):
            return int(raw) - 1
        warn("Invalid choice, try again.")

# ── Shell helpers ─────────────────────────────────────────────────────────────

def run(cmd, check=True, extra_env=None, cwd=None):
    """Run a command list, optionally merging extra environment variables."""
    display = " ".join(str(x) for x in cmd)
    print(c(f"  $ {display}", CYAN))
    merged_env = {**os.environ, **(extra_env or {})}
    result = subprocess.run(cmd, env=merged_env, cwd=cwd)
    if check and result.returncode != 0:
        error(f"Command failed (exit {result.returncode})")
        sys.exit(result.returncode)
    return result.returncode

def check_docker():
    if shutil.which("docker") is None:
        error("Docker not found. Install it from https://docs.docker.com/get-docker/")
        sys.exit(1)
    r = subprocess.run(["docker", "compose", "version"], capture_output=True)
    if r.returncode != 0:
        error("Docker Compose v2 not found. Please upgrade Docker.")
        sys.exit(1)
    success("Docker & Docker Compose detected.")

def check_diesel():
    if shutil.which("diesel") is not None:
        success("diesel CLI detected.")
        return
    error("diesel CLI not found. Install it with the command below, then re-run this script.\n")
    if sys.platform == "win32":
        print(c(
            "  Set-ExecutionPolicy RemoteSigned -scope CurrentUser\n"
            "  irm https://github.com/diesel-rs/diesel/releases/download/v2.3.5/diesel_cli-installer.ps1 | iex",
            CYAN,
        ))
    else:
        print(c(
            "  curl --proto '=https' --tlsv1.2 -LsSf"
            " https://github.com/diesel-rs/diesel/releases/download/v2.3.5/diesel_cli-installer.sh | sh",
            CYAN,
        ))
    print()
    sys.exit(1)

# ── .env ──────────────────────────────────────────────────────────────────────

ENV_FILE     = Path(".env")
COMPOSE_FILE = Path("docker-compose.yml")
DB_SCHEMA    = Path("db_schema")

POSTGRES_CONTAINER = "postgres-container"
APP_NETWORK        = "app-network"

def load_env():
    env = {}
    if ENV_FILE.exists():
        for line in ENV_FILE.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if line and not line.startswith("#") and "=" in line:
                k, _, v = line.partition("=")
                env[k.strip()] = v.strip()
    return env

def save_env(env):
    lines = [f"{k}={v}" for k, v in env.items()]
    ENV_FILE.write_text("\n".join(lines) + "\n", encoding="utf-8")
    success(".env saved.")

# ── DB URL helpers ────────────────────────────────────────────────────────────

def is_remote_db(env):
    """Return True when DB_IP is set, meaning postgres lives on a remote host."""
    return bool(env.get("DB_IP", "").strip())

def database_url(env):
    """
    Build the DATABASE_URL used inside docker-compose services.
    - Remote DB  → uses DB_IP / DB_PORT from .env
    - Local DB   → uses the postgres container hostname
    """
    user = env["POSTGRES_USER"]
    pw   = env["POSTGRES_PASSWORD"]
    db   = env["POSTGRES_DB"]
    if is_remote_db(env):
        host = env["DB_IP"]
        port = env.get("DB_PORT", "5432")
        return f"postgres://{user}:{pw}@{host}:{port}/{db}"
    return f"postgres://{user}:{pw}@{POSTGRES_CONTAINER}/{db}"

def migration_url(env):
    """
    Build the DATABASE_URL used by the diesel CLI running on the host machine.
    - Remote DB  → same as database_url (the remote host is already reachable)
    - Local DB   → 127.0.0.1:5432 (the container port mapped to the host)
    """
    user = env["POSTGRES_USER"]
    pw   = env["POSTGRES_PASSWORD"]
    db   = env["POSTGRES_DB"]
    if is_remote_db(env):
        host = env["DB_IP"]
        port = env.get("DB_PORT", "5432")
        return f"postgres://{user}:{pw}@{host}:{port}/{db}"
    return f"postgres://{user}:{pw}@127.0.0.1:5432/{db}"

# ── docker-compose builder ────────────────────────────────────────────────────

def _postgres_svc(env):
    return {
        "image": "postgres",
        "container_name": POSTGRES_CONTAINER,
        "environment": {
            "POSTGRES_USER":     env["POSTGRES_USER"],
            "POSTGRES_PASSWORD": env["POSTGRES_PASSWORD"],
            "POSTGRES_DB":       env["POSTGRES_DB"],
        },
        "healthcheck": {
            "test":     ["CMD-SHELL", f"pg_isready -U {env['POSTGRES_USER']} -d {env['POSTGRES_DB']}"],
            "interval": "5s",
            "timeout":  "5s",
            "retries":  5,
        },
        "networks": [APP_NETWORK],
        "volumes":  ["postgres-data:/var/lib/postgresql"],
        "ports":    ["127.0.0.1:5432:5432/tcp"],
        "restart":  "unless-stopped",
    }

def _worker_svc(env, local_postgres):
    svc = {
        "image": "ghcr.io/kauri-off/mine_search/worker:latest",
        "environment": {
            "THREADS":                env["THREADS"],
            "DATABASE_URL":           database_url(env),
            "SEARCH_MODULE":          env["SEARCH_MODULE"],
            "UPDATE_MODULE":          env["UPDATE_MODULE"],
            "UPDATE_WITH_CONNECTION": env["UPDATE_WITH_CONNECTION"],
            "RUST_LOG":               env["RUST_LOG"],
            "ONLY_UPDATE_SPOOFABLE":  env["ONLY_UPDATE_SPOOFABLE"]
        },
        "networks": [APP_NETWORK],
        "restart":  "unless-stopped",
    }
    if local_postgres:
        svc["depends_on"] = {"postgres": {"condition": "service_healthy"}}
    return svc

def _backend_svc(env, local_postgres):
    svc = {
        "image": "ghcr.io/kauri-off/mine_search/backend:latest",
        "environment": {
            "DATABASE_URL":     database_url(env),
            "BACKEND_PASSWORD": env["BACKEND_PASSWORD"],
        },
        "networks": [APP_NETWORK],
        "restart":  "unless-stopped",
    }
    if local_postgres:
        svc["depends_on"] = {"postgres": {"condition": "service_healthy"}}
    return svc

def _frontend_svc():
    return {
        "image": "ghcr.io/kauri-off/mine_search/frontend:latest",
        "depends_on": ["backend"],
        "networks":   [APP_NETWORK],
        "restart":    "unless-stopped",
    }

def _nginx_svc():
    return {
        "image":          "nginx:alpine",
        "container_name": "nginx-proxy",
        "volumes":        ["./nginx.conf:/etc/nginx/conf.d/default.conf:ro"],
        "ports":          ["8080:80"],
        "depends_on":     ["backend", "frontend"],
        "networks":       [APP_NETWORK],
        "restart":        "unless-stopped",
    }

def build_compose(services, env):
    """Build a docker-compose structure from scratch using env for all values."""
    local_postgres = "postgres" in services

    compose = {
        "services": {},
        "networks": {APP_NETWORK: {}},
        "volumes":  {},
    }

    if local_postgres:
        compose["services"]["postgres"] = _postgres_svc(env)
        compose["volumes"]["postgres-data"] = {}

    if "worker" in services:
        compose["services"]["worker"] = _worker_svc(env, local_postgres)

    if "backend" in services:
        compose["services"]["backend"] = _backend_svc(env, local_postgres)

    if "frontend" in services:
        if "backend" not in services:
            warn("frontend depends on backend, but backend is not selected.")
        compose["services"]["frontend"] = _frontend_svc()

    if "nginx" in services:
        compose["services"]["nginx"] = _nginx_svc()

    return compose

def save_compose(data):
    with open(COMPOSE_FILE, "w", encoding="utf-8") as f:
        yaml.dump(data, f, default_flow_style=False, sort_keys=False, allow_unicode=True)
    success("docker-compose.yml written.")

# ── Service / profile selection ───────────────────────────────────────────────

ALL_SERVICES = ["postgres", "worker", "backend", "frontend", "nginx"]

PROFILES = {
    "Full (all services)":       ["postgres", "worker", "backend", "frontend", "nginx"],
    "Worker only":               ["worker"],
    "DB only":                   ["postgres"],
    "Everything without worker": ["postgres", "backend", "frontend", "nginx"],
    "Custom":                    None,
}

def pick_services():
    idx  = menu("Select installation profile:", list(PROFILES.keys()))
    name = list(PROFILES.keys())[idx]
    svcs = PROFILES[name]

    if svcs is None:
        svcs = []
        print("\n  Pick services to include:")
        for svc in ALL_SERVICES:
            if ask_yes(f"    Include '{svc}'?", default=True):
                svcs.append(svc)

    if not svcs:
        error("No services selected.")
        sys.exit(1)

    success(f"Profile : {name}")
    print(   f"  Services: {', '.join(svcs)}")
    return svcs

# ── Diesel migrations ─────────────────────────────────────────────────────────

def wait_for_postgres(pg_user, retries=20, delay=3):
    """Poll the local postgres container until it accepts connections."""
    print(f"\n  Waiting for postgres", end="", flush=True)
    for _ in range(retries):
        r = subprocess.run(
            ["docker", "compose", "exec", "postgres", "pg_isready", "-U", pg_user],
            capture_output=True,
        )
        if r.returncode == 0:
            print(c("  ready!", GREEN))
            return True
        print(".", end="", flush=True)
        time.sleep(delay)
    print()
    return False

def run_diesel_migrations(env):
    """Run diesel migrations from db_schema/ using migration_url derived from env."""
    header("DIESEL MIGRATIONS")

    if not DB_SCHEMA.is_dir():
        warn(f"'{DB_SCHEMA}/' directory not found — skipping migrations.")
        return

    check_diesel()

    url = migration_url(env)
    run(
        ["diesel", "migration", "run"],
        extra_env={"DATABASE_URL": url},
        cwd=str(DB_SCHEMA),
    )
    success("Diesel migrations applied.")

# ── Mode: Install ─────────────────────────────────────────────────────────────

def mode_install():
    header("INSTALL")
    check_docker()

    services       = pick_services()
    local_postgres = "postgres" in services
    needs_backend  = "backend" in services
    needs_db       = any(s in services for s in ("worker", "backend"))

    env = load_env()
    print("\n  Configure environment variables:")

    # Core DB credentials
    env["POSTGRES_USER"]     = ask("POSTGRES_USER",     env.get("POSTGRES_USER",     "user"))
    env["POSTGRES_PASSWORD"] = ask("POSTGRES_PASSWORD", env.get("POSTGRES_PASSWORD", "CHANGE_THIS"))
    env["POSTGRES_DB"]       = ask("POSTGRES_DB",       env.get("POSTGRES_DB",       "mine_search_db"))

    # Remote DB overrides (optional — leave blank for local)
    if not local_postgres and needs_db:
        warn("No local postgres selected. Enter remote DB connection details.")
        env["DB_IP"]   = ask("DB_IP   (remote host)",  env.get("DB_IP",   ""))
        env["DB_PORT"] = ask("DB_PORT (remote port)",  env.get("DB_PORT", "5432"))
    else:
        # Clear remote keys if switching back to local
        env.pop("DB_IP",   None)
        env.pop("DB_PORT", None)

    # App secrets
    if needs_backend:
        env["BACKEND_PASSWORD"] = ask("BACKEND_PASSWORD", env.get("BACKEND_PASSWORD", "CHANGE_THIS"))

    # Worker tuning
    if "worker" in services:
        env["THREADS"]                = ask("THREADS",                env.get("THREADS",                "500"))
        env["SEARCH_MODULE"]          = ask("SEARCH_MODULE",          env.get("SEARCH_MODULE",          "true"))
        env["UPDATE_MODULE"]          = ask("UPDATE_MODULE",          env.get("UPDATE_MODULE",          "true"))
        env["UPDATE_WITH_CONNECTION"] = ask("UPDATE_WITH_CONNECTION", env.get("UPDATE_WITH_CONNECTION", "false"))
        env["RUST_LOG"]               = ask("RUST_LOG (info/debug)",  env.get("RUST_LOG",               "info"))
        env["ONLY_UPDATE_SPOOFABLE"]  = ask("ONLY_UPDATE_SPOOFABLE",  env.get("RUST_LOG",               "false"))

    # Persist settings and generate compose
    save_env(env)
    save_compose(build_compose(services, env))

    # Pull images
    run(["docker", "compose", "pull"])

    # Start postgres first (if local), migrate, then bring up everything
    if needs_db:
        if ask_yes("\nRun diesel(postgres) migrations?", default=True):
            if local_postgres:
                run(["docker", "compose", "up", "postgres", "-d"])
                if wait_for_postgres(env["POSTGRES_USER"]):
                    run_diesel_migrations(env)
                else:
                    warn("Postgres did not become ready in time.")
            else:
                    run_diesel_migrations(env)

    if ask_yes("\nRun docker compose up?", default=True):
        run(["docker", "compose", "up", "-d"])

    success("Installation complete.")

# ── Mode: Update ──────────────────────────────────────────────────────────────

def mode_update():
    header("UPDATE")
    check_docker()

    choice = menu("What would you like to update?", [
        "Pull latest images & restart all services",
        "Pull latest images & restart one service",
        "Run diesel migrations",
        "Start everything",
        "Stop everything"
    ])

    if choice == 0:
        run(["docker", "compose", "down"])
        run(["docker", "compose", "pull"])
        run(["docker", "compose", "up", "-d"])
        success("All services updated.")
    elif choice == 1:
        svc = ask("Service name (worker / backend / frontend / nginx)")
        run(["docker", "compose", "down", svc])
        run(["docker", "compose", "pull", svc])
        run(["docker", "compose", "up", svc, "-d"])
        success(f"'{svc}' updated.")
    elif choice == 2:
        run_diesel_migrations(load_env())
    elif choice == 3:
        run(["docker", "compose", "up", "-d"])
    elif choice == 4:
        run(["docker", "compose", "down"])

# ── Mode: Change Settings ─────────────────────────────────────────────────────

def mode_settings():
    header("CHANGE SETTINGS")

    env = load_env()
    if not env:
        warn(".env not found or empty — starting from scratch.")

    print("  Press Enter to keep the current value.\n")

    fields = [
        ("POSTGRES_USER",          "PostgreSQL user"),
        ("POSTGRES_PASSWORD",      "PostgreSQL password"),
        ("POSTGRES_DB",            "PostgreSQL database name"),
        ("DB_IP",                  "Remote DB host (leave blank for local Docker)"),
        ("DB_PORT",                "Remote DB port"),
        ("BACKEND_PASSWORD",       "Backend API password"),
        ("THREADS",                "Worker threads"),
        ("SEARCH_MODULE",          "Search module enabled (true/false)"),
        ("UPDATE_MODULE",          "Update module enabled (true/false)"),
        ("UPDATE_WITH_CONNECTION", "Update with connection (true/false)"),
        ("ONLY_UPDATE_SPOOFABLE",  "Only updates servers with flag Spoofable"),
        ("RUST_LOG",               "Worker log level (info/debug)"),
    ]

    changed = False
    for key, label in fields:
        current = env.get(key, "")
        new_val = ask(label, current)
        if new_val != current:
            if new_val == "":
                env.pop(key, None)  # remove key entirely when cleared
            else:
                env[key] = new_val
            changed = True

    if not changed:
        print("\n  No changes made.")
        return

    save_env(env)

    # Regenerate docker-compose so embedded URLs reflect the new values
    if COMPOSE_FILE.exists():
        with open(COMPOSE_FILE, encoding="utf-8") as f:
            existing = yaml.safe_load(f) or {}
        services = list(existing.get("services", {}).keys())
        if services:
            save_compose(build_compose(services, env))

    if ask_yes("\nRestart services to apply new settings?", default=True):
        check_docker()
        run(["docker", "compose", "up", "-d"])
        success("Services restarted with new settings.")

# ── Main ──────────────────────────────────────────────────────────────────────

def main():
    print(f"\n{BOLD}{CYAN}  mine_search  —  Setup Script{RESET}")

    if len(sys.argv) > 1:
        arg = sys.argv[1].lower()
        dispatch = {
            "i": mode_install,  "install":  mode_install,
            "u": mode_update,   "update":   mode_update,
            "s": mode_settings, "settings": mode_settings,
        }
        fn = dispatch.get(arg)
        if fn:
            fn()
            return
        error(f"Unknown argument '{arg}'. Valid: install (i), update (u), settings (s)")
        sys.exit(1)

    choice = menu("Select mode:", [
        "Install  — fresh setup, choose which modules to deploy",
        "Update   — pull latest images, run migrations, rebuild",
        "Settings — change .env values and restart",
        "Exit",
    ])

    [mode_install, mode_update, mode_settings, lambda: print("  Bye!\n")][choice]()

if __name__ == "__main__":
    main()