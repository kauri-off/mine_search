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

# ── Auto-install PyYAML ───────────────────────────────────────────────────────
try:
    import yaml
except ImportError:
    print("PyYAML not found. Installing...")
    subprocess.check_call([sys.executable, "-m", "pip", "install", "pyyaml"])
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

def _worker_svc(database_url, env, local_postgres):
    svc = {
        "image": "ghcr.io/kauri-off/mine_search/worker:latest",
        "environment": {
            "THREADS":                env["THREADS"],
            "DATABASE_URL":           database_url,
            "SEARCH_MODULE":          env["SEARCH_MODULE"],
            "UPDATE_MODULE":          env["UPDATE_MODULE"],
            "UPDATE_WITH_CONNECTION": env["UPDATE_WITH_CONNECTION"],
            "RUST_LOG":               "debug",
        },
        "networks": [APP_NETWORK],
        "restart":  "unless-stopped",
    }
    if local_postgres:
        svc["depends_on"] = {"postgres": {"condition": "service_healthy"}}
    return svc

def _backend_svc(database_url, env, local_postgres):
    svc = {
        "image": "ghcr.io/kauri-off/mine_search/backend:latest",
        "environment": {
            "DATABASE_URL":     database_url,
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
        "build":      {"context": ".", "dockerfile": "frontend.Dockerfile"},
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

def build_compose(services, database_url, env):
    """
    Build a docker-compose structure from scratch.

    services     – list of service names to include
    database_url – fully resolved postgres connection string
    env          – settings dict
    """
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
        compose["services"]["worker"] = _worker_svc(database_url, env, local_postgres)

    if "backend" in services:
        compose["services"]["backend"] = _backend_svc(database_url, env, local_postgres)

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

# ── DB configuration ──────────────────────────────────────────────────────────

def collect_local_db_env(env):
    env["POSTGRES_USER"]     = ask("POSTGRES_USER",     env.get("POSTGRES_USER",     "user"))
    env["POSTGRES_PASSWORD"] = ask("POSTGRES_PASSWORD", env.get("POSTGRES_PASSWORD", "CHANGE_THIS"))
    env["POSTGRES_DB"]       = ask("POSTGRES_DB",       env.get("POSTGRES_DB",       "mine_search_db"))
    # URL used inside docker-compose (container-to-container)
    container_url = (
        f"postgres://{env['POSTGRES_USER']}:{env['POSTGRES_PASSWORD']}"
        f"@{POSTGRES_CONTAINER}/{env['POSTGRES_DB']}"
    )
    # URL used from the host machine (diesel CLI connects via mapped port)
    localhost_url = (
        f"postgres://{env['POSTGRES_USER']}:{env['POSTGRES_PASSWORD']}"
        f"@127.0.0.1:5432/{env['POSTGRES_DB']}"
    )
    return env, container_url, localhost_url

def collect_remote_db_env(env):
    print("\n  Enter remote PostgreSQL connection details:")
    env["POSTGRES_USER"]     = ask("DB user",     env.get("POSTGRES_USER",     "user"))
    env["POSTGRES_PASSWORD"] = ask("DB password", env.get("POSTGRES_PASSWORD", "CHANGE_THIS"))
    env["POSTGRES_DB"]       = ask("DB name",     env.get("POSTGRES_DB",       "mine_search_db"))
    host                     = ask("DB host",     "db.example.com")
    port                     = ask("DB port",     "5432")
    url = (
        f"postgres://{env['POSTGRES_USER']}:{env['POSTGRES_PASSWORD']}"
        f"@{host}:{port}/{env['POSTGRES_DB']}"
    )
    return env, url

# ── Diesel migrations ─────────────────────────────────────────────────────────

def wait_for_postgres(pg_user, retries=20, delay=3):
    """Poll the local postgres container until it accepts connections."""
    print(f"\n  Waiting for postgres", end="", flush=True)
    for _ in range(retries):
        r = subprocess.run(
            ["docker", "compose", "exec", "postgres",
             "pg_isready", "-U", pg_user],
            capture_output=True,
        )
        if r.returncode == 0:
            print(c("  ready!", GREEN))
            return True
        print(".", end="", flush=True)
        time.sleep(delay)
    print()
    return False

def run_diesel_migrations(database_url, localhost_url=None):
    """
    Run diesel migrations from the db_schema/ directory.

    database_url  – the URL stored in docker-compose (uses container hostname).
    localhost_url – URL reachable from the host machine (127.0.0.1:5432).
                   Diesel runs on the host, so it must use localhost_url when
                   postgres is a local Docker container.
    """
    header("DIESEL MIGRATIONS")

    if not DB_SCHEMA.is_dir():
        warn(f"'{DB_SCHEMA}/' directory not found — skipping migrations.")
        return

    check_diesel()

    migration_url = localhost_url if localhost_url else database_url
    run(
        ["diesel", "migration", "run"],
        extra_env={"DATABASE_URL": migration_url},
        cwd=str(DB_SCHEMA),
    )
    success("Diesel migrations applied.")

# ── Mode: Install ─────────────────────────────────────────────────────────────

def mode_install():
    header("INSTALL")
    check_docker()

    services       = pick_services()
    local_postgres = "postgres" in services
    needs_db       = any(s in services for s in ("worker", "backend"))

    env = load_env()
    print("\n  Configure environment variables:")

    # DB connection
    if local_postgres:
        env, database_url, localhost_url = collect_local_db_env(env)
    elif needs_db:
        warn("No local postgres selected but worker/backend need a database.")
        env, database_url = collect_remote_db_env(env)
        localhost_url = None
    else:
        database_url = ""
        localhost_url = None

    # App secrets
    if needs_db:
        env["BACKEND_PASSWORD"] = ask("BACKEND_PASSWORD", env.get("BACKEND_PASSWORD", "CHANGE_THIS"))

    # Worker tuning
    if "worker" in services:
        env["THREADS"]                = ask("THREADS",                env.get("THREADS",                "500"))
        env["SEARCH_MODULE"]          = ask("SEARCH_MODULE",          env.get("SEARCH_MODULE",          "true"))
        env["UPDATE_MODULE"]          = ask("UPDATE_MODULE",          env.get("UPDATE_MODULE",          "true"))
        env["UPDATE_WITH_CONNECTION"] = ask("UPDATE_WITH_CONNECTION", env.get("UPDATE_WITH_CONNECTION", "false"))

    # Persist settings
    save_env(env)
    compose_data = build_compose(services, database_url, env)
    save_compose(compose_data)

    # Pull images
    if ask_yes("\nPull latest Docker images?", default=True):
        run(["docker", "compose", "pull"])

    # Start postgres first, migrate, then bring up everything else
    if needs_db:
        if local_postgres:
            run(["docker", "compose", "up", "postgres", "-d"])
            pg_user = env.get("POSTGRES_USER", "user")
            if wait_for_postgres(pg_user):
                run_diesel_migrations(localhost_url)
            else:
                warn("Postgres did not become ready in time.")
                warn(f"Run manually: cd {DB_SCHEMA} && DATABASE_URL='{localhost_url}' diesel migration run")
        else:
            if ask_yes("\nRun diesel migrations against the remote DB now?", default=True):
                run_diesel_migrations(database_url)

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
        "Rebuild frontend image",
        "Everything (pull + migrate + rebuild frontend)",
    ])

    if choice in (0, 4):
        run(["docker", "compose", "pull"])
        run(["docker", "compose", "up", "-d", "--remove-orphans"])
        success("All services updated.")

    if choice == 1:
        svc = ask("Service name (worker / backend / frontend / nginx)")
        run(["docker", "compose", "pull", svc])
        run(["docker", "compose", "up", "-d", svc])
        success(f"'{svc}' updated.")

    if choice in (2, 4):
        env = load_env()
        # Build DATABASE_URL from saved .env values
        pg_user = env.get("POSTGRES_USER", "user")
        pg_pass = env.get("POSTGRES_PASSWORD", "")
        pg_db   = env.get("POSTGRES_DB", "mine_search_db")
        database_url = f"postgres://{pg_user}:{pg_pass}@{POSTGRES_CONTAINER}/{pg_db}"
        run_diesel_migrations(database_url)

    if choice in (3, 4):
        run(["docker", "compose", "build", "frontend"])
        run(["docker", "compose", "up", "-d", "frontend"])
        success("Frontend rebuilt and restarted.")

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
        ("BACKEND_PASSWORD",       "Backend API password"),
        ("THREADS",                "Worker threads"),
        ("SEARCH_MODULE",          "Search module enabled (true/false)"),
        ("UPDATE_MODULE",          "Update module enabled (true/false)"),
        ("UPDATE_WITH_CONNECTION", "Update with connection (true/false)"),
    ]

    changed = False
    for key, label in fields:
        current = env.get(key, "")
        new_val = ask(label, current)
        if new_val != current:
            env[key] = new_val
            changed = True

    if not changed:
        print("\n  No changes made.")
        return

    save_env(env)

    # Regenerate docker-compose with updated values
    if COMPOSE_FILE.exists():
        with open(COMPOSE_FILE, encoding="utf-8") as f:
            existing = yaml.safe_load(f) or {}
        services = list(existing.get("services", {}).keys())
        if services:
            local_postgres = "postgres" in services
            pg_user = env.get("POSTGRES_USER", "user")
            pg_pass = env.get("POSTGRES_PASSWORD", "")
            pg_db   = env.get("POSTGRES_DB", "mine_search_db")
            if local_postgres:
                database_url = f"postgres://{pg_user}:{pg_pass}@{POSTGRES_CONTAINER}/{pg_db}"
            else:
                # Try to reconstruct from existing compose backend env
                backend_env = existing.get("services", {}).get("backend", {}).get("environment", {})
                database_url = backend_env.get("DATABASE_URL", "")
            compose_data = build_compose(services, database_url, env)
            save_compose(compose_data)

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