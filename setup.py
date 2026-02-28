#!/usr/bin/env python3
"""
mine_search — Setup Script
Modes: Install / Update / Settings
Platform independent (Windows, Linux, macOS)

Architecture note:
  • All runtime values live in .env (single source of truth).
  • docker-compose.yml references them via ${VAR} substitution — values
    are never baked into the YAML.  Editing .env is enough to change
    behaviour without touching the compose file.
"""

from __future__ import annotations

import os
import secrets
import sys
import time
import shutil
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import yaml

# ── ANSI colours ──────────────────────────────────────────────────────────────

RESET  = "\033[0m"
BOLD   = "\033[1m"
CYAN   = "\033[96m"
GREEN  = "\033[92m"
YELLOW = "\033[93m"
RED    = "\033[91m"


def _c(text: str, col: str) -> str:
    return f"{col}{text}{RESET}"

def header(text: str) -> None:
    print(f"\n{BOLD}{CYAN}{'─'*52}\n  {text}\n{'─'*52}{RESET}")

def success(text: str) -> None:
    print(_c(f"  ✔  {text}", GREEN))

def warn(text: str) -> None:
    print(_c(f"  ⚠  {text}", YELLOW))

def error(text: str) -> None:
    print(_c(f"  ✖  {text}", RED))


# ── UI helpers ────────────────────────────────────────────────────────────────

def ask(prompt: str, default: str = "") -> str:
    suffix = f" [{default}]" if default else ""
    val = input(f"  {prompt}{suffix}: ").strip()
    return val if val else default


def ask_yes(prompt: str, default: bool = True) -> bool:
    suffix = "Y/n" if default else "y/N"
    val = input(f"  {prompt} [{suffix}]: ").strip().lower()
    if val == "":
        return default
    return val in ("y", "yes")


def menu(title: str, options: list[str]) -> int:
    print(f"\n  {BOLD}{title}{RESET}")
    for i, opt in enumerate(options, 1):
        print(f"    {i}) {opt}")
    while True:
        raw = input("  Choose: ").strip()
        if raw.isdigit() and 1 <= int(raw) <= len(options):
            return int(raw) - 1
        warn("Invalid choice, try again.")


# ── Shell / prerequisite checks ───────────────────────────────────────────────

class Checks:
    """Static prerequisite checks that abort on failure."""

    @staticmethod
    def docker() -> None:
        if shutil.which("docker") is None:
            error("Docker not found. Install it from https://docs.docker.com/get-docker/")
            sys.exit(1)
        r = subprocess.run(["docker", "compose", "version"], capture_output=True)
        if r.returncode != 0:
            error("Docker Compose v2 not found. Please upgrade Docker.")
            sys.exit(1)
        success("Docker & Docker Compose detected.")

    @staticmethod
    def diesel() -> None:
        if shutil.which("diesel") is not None:
            success("diesel CLI detected.")
            return
        error("diesel CLI not found. Install it with the command below, then re-run.\n")
        if sys.platform == "win32":
            print(_c(
                "  Set-ExecutionPolicy RemoteSigned -scope CurrentUser\n"
                "  irm https://github.com/diesel-rs/diesel/releases/download/v2.3.5"
                "/diesel_cli-installer.ps1 | iex",
                CYAN,
            ))
        else:
            print(_c(
                "  curl --proto '=https' --tlsv1.2 -LsSf"
                " https://github.com/diesel-rs/diesel/releases/download/v2.3.5"
                "/diesel_cli-installer.sh | sh",
                CYAN,
            ))
        print()
        sys.exit(1)

    @staticmethod
    def git() -> None:
        if shutil.which("git") is None:
            error("git not found — required before running migrations.")
            sys.exit(1)


def run(
    cmd: list[str],
    *,
    check: bool = True,
    extra_env: dict[str, str] | None = None,
    cwd: str | None = None,
) -> int:
    display = " ".join(str(x) for x in cmd)
    print(_c(f"  $ {display}", CYAN))
    merged_env: dict[str, str] = {**os.environ, **(extra_env or {})}
    result = subprocess.run(cmd, env=merged_env, cwd=cwd)
    if check and result.returncode != 0:
        error(f"Command failed (exit {result.returncode})")
        sys.exit(result.returncode)
    return result.returncode


# ── Configuration model ───────────────────────────────────────────────────────

@dataclass
class DatabaseConfig:
    user:     str
    password: str
    db:       str
    host:     str = ""   # empty → local Docker container
    port:     str = "5432"

    @property
    def is_remote(self) -> bool:
        return bool(self.host.strip())

    def service_url(self, local_container: str) -> str:
        """URL used by services inside Docker (container hostname or remote host)."""
        h = self.host if self.is_remote else local_container
        p = f":{self.port}" if self.is_remote else ""
        return f"postgres://{self.user}:{self.password}@{h}{p}/{self.db}"

    def migration_url(self) -> str:
        """URL used by diesel CLI running on the host machine."""
        h = self.host if self.is_remote else "127.0.0.1"
        return f"postgres://{self.user}:{self.password}@{h}:{self.port}/{self.db}"


@dataclass
class WorkerConfig:
    threads:                str = "500"
    search_module:          str = "true"
    update_module:          str = "true"
    update_with_connection: str = "false"
    rust_log:               str = "info"
    only_update_spoofable:  str = "false"
    only_update_cracked:    str = "false"


@dataclass
class AppConfig:
    db:               DatabaseConfig
    backend_password: str               = "CHANGE_THIS"
    worker:           WorkerConfig      = field(default_factory=WorkerConfig)
    services:         list[str]         = field(default_factory=list)
    jwt_secret:       str               = field(default_factory=lambda: secrets.token_hex(16))

    # ── .env serialisation ────────────────────────────────────────────────────

    def to_env(self) -> dict[str, str]:
        d: dict[str, str] = {
            "POSTGRES_USER":            self.db.user,
            "POSTGRES_PASSWORD":        self.db.password,
            "POSTGRES_DB":              self.db.db,
            "POSTGRES_PORT":            self.db.port,
            "BACKEND_PASSWORD":         self.backend_password,
            "BACKEND_JWT_SECRET":       self.jwt_secret,
            "THREADS":                  self.worker.threads,
            "SEARCH_MODULE":            self.worker.search_module,
            "UPDATE_MODULE":            self.worker.update_module,
            "UPDATE_WITH_CONNECTION":   self.worker.update_with_connection,
            "RUST_LOG":                 self.worker.rust_log,
            "ONLY_UPDATE_SPOOFABLE":    self.worker.only_update_spoofable,
            "ONLY_UPDATE_CRACKED":      self.worker.only_update_cracked,
        }
        if self.db.is_remote:
            d["DB_HOST"] = self.db.host
        return d

    @classmethod
    def from_env(cls, raw: dict[str, str]) -> "AppConfig":
        db = DatabaseConfig(
            user     = raw.get("POSTGRES_USER",     "user"),
            password = raw.get("POSTGRES_PASSWORD", "CHANGE_THIS"),
            db       = raw.get("POSTGRES_DB",       "mine_search_db"),
            host     = raw.get("DB_HOST",           ""),
            port     = raw.get("POSTGRES_PORT",     "5432"),
        )
        worker = WorkerConfig(
            threads                = raw.get("THREADS",                "500"),
            search_module          = raw.get("SEARCH_MODULE",          "true"),
            update_module          = raw.get("UPDATE_MODULE",          "true"),
            update_with_connection = raw.get("UPDATE_WITH_CONNECTION", "false"),
            rust_log               = raw.get("RUST_LOG",               "info"),
            only_update_spoofable  = raw.get("ONLY_UPDATE_SPOOFABLE",  "false"),
            only_update_cracked    = raw.get("ONLY_UPDATE_CRACKED",    "false"),
        )
        return cls(
            db=db,
            backend_password=raw.get("BACKEND_PASSWORD", "CHANGE_THIS"),
            worker=worker,
            jwt_secret=raw.get("BACKEND_JWT_SECRET") or secrets.token_hex(16),
        )


# ── .env persistence ──────────────────────────────────────────────────────────

ENV_FILE = Path(".env")


def load_raw_env() -> dict[str, str]:
    env: dict[str, str] = {}
    if ENV_FILE.exists():
        for line in ENV_FILE.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if line and not line.startswith("#") and "=" in line:
                k, _, v = line.partition("=")
                env[k.strip()] = v.strip()
    return env


def save_env(cfg: AppConfig) -> None:
    lines = [f"{k}={v}" for k, v in cfg.to_env().items()]
    ENV_FILE.write_text("\n".join(lines) + "\n", encoding="utf-8")
    success(".env saved.")


# ── docker-compose builder ────────────────────────────────────────────────────
#
# Values are referenced via ${VAR} — docker compose reads .env automatically.
# This means you never have to regenerate docker-compose.yml just because a
# password changed: edit .env, restart, done.

POSTGRES_CONTAINER = "postgres-container"
APP_NETWORK        = "app-network"
COMPOSE_FILE       = Path("docker-compose.yml")
DB_SCHEMA          = Path("db_schema")


class ComposeBuilder:
    """Builds a docker-compose structure whose runtime values come from .env."""

    def __init__(self, services: list[str]) -> None:
        self.services = services

    @property
    def _local_postgres(self) -> bool:
        return "postgres" in self.services

    # ── service definitions ───────────────────────────────────────────────────

    def _postgres_svc(self) -> dict[str, Any]:
        return {
            "image":          "postgres",
            "container_name": POSTGRES_CONTAINER,
            "environment": {
                "POSTGRES_USER":     "${POSTGRES_USER}",
                "POSTGRES_PASSWORD": "${POSTGRES_PASSWORD}",
                "POSTGRES_DB":       "${POSTGRES_DB}",
            },
            "healthcheck": {
                "test":     ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER} -d ${POSTGRES_DB}"],
                "interval": "5s",
                "timeout":  "5s",
                "retries":  5,
            },
            "networks": [APP_NETWORK],
            "volumes":  ["postgres-data:/var/lib/postgresql"],
            "ports":    ["127.0.0.1:5432:5432/tcp"],
            "restart":  "unless-stopped",
        }

    def _worker_svc(self) -> dict[str, Any]:
        svc: dict[str, Any] = {
            "image": "ghcr.io/kauri-off/mine_search/worker:latest",
            "environment": {
                "THREADS":                "${THREADS}",
                "DATABASE_URL":           "${DATABASE_URL}",
                "SEARCH_MODULE":          "${SEARCH_MODULE}",
                "UPDATE_MODULE":          "${UPDATE_MODULE}",
                "UPDATE_WITH_CONNECTION": "${UPDATE_WITH_CONNECTION}",
                "RUST_LOG":               "${RUST_LOG}",
                "ONLY_UPDATE_SPOOFABLE":  "${ONLY_UPDATE_SPOOFABLE}",
            },
            "networks": [APP_NETWORK],
            "restart":  "unless-stopped",
        }
        if self._local_postgres:
            svc["depends_on"] = {"postgres": {"condition": "service_healthy"}}
        return svc

    def _backend_svc(self) -> dict[str, Any]:
        svc: dict[str, Any] = {
            "image": "ghcr.io/kauri-off/mine_search/backend:latest",
            "environment": {
                "DATABASE_URL":       "${DATABASE_URL}",
                "BACKEND_PASSWORD":   "${BACKEND_PASSWORD}",
                "BACKEND_JWT_SECRET": "${BACKEND_JWT_SECRET}",
            },
            "networks": [APP_NETWORK],
            "restart":  "unless-stopped",
        }
        if self._local_postgres:
            svc["depends_on"] = {"postgres": {"condition": "service_healthy"}}
        return svc

    def _frontend_svc(self) -> dict[str, Any]:
        return {
            "image":      "ghcr.io/kauri-off/mine_search/frontend:latest",
            "depends_on": ["backend"],
            "networks":   [APP_NETWORK],
            "restart":    "unless-stopped",
        }

    def _nginx_svc(self) -> dict[str, Any]:
        return {
            "image":          "nginx:alpine",
            "container_name": "nginx-proxy",
            "volumes":        ["./nginx.conf:/etc/nginx/conf.d/default.conf:ro"],
            "ports":          ["8080:80"],
            "depends_on":     ["backend", "frontend"],
            "networks":       [APP_NETWORK],
            "restart":        "unless-stopped",
        }

    # ── public API ────────────────────────────────────────────────────────────

    def build(self) -> dict[str, Any]:
        compose: dict[str, Any] = {
            "services": {},
            "networks": {APP_NETWORK: {}},
            "volumes":  {},
        }

        builders: dict[str, Any] = {
            "postgres": self._postgres_svc,
            "worker":   self._worker_svc,
            "backend":  self._backend_svc,
            "frontend": self._frontend_svc,
            "nginx":    self._nginx_svc,
        }

        for svc in self.services:
            if svc == "frontend" and "backend" not in self.services:
                warn("frontend depends on backend, but backend is not selected.")
            fn = builders.get(svc)
            if fn:
                compose["services"][svc] = fn()

        if self._local_postgres:
            compose["volumes"]["postgres-data"] = {}

        return compose

    def save(self) -> None:
        with open(COMPOSE_FILE, "w", encoding="utf-8") as f:
            yaml.dump(self.build(), f, default_flow_style=False, sort_keys=False, allow_unicode=True)
        success("docker-compose.yml written.")

    @staticmethod
    def active_services() -> list[str]:
        """Return the list of services defined in the existing compose file."""
        if not COMPOSE_FILE.exists():
            return []
        with open(COMPOSE_FILE, encoding="utf-8") as f:
            data = yaml.safe_load(f) or {}
        return list(data.get("services", {}).keys())


# ── Service / profile selection ───────────────────────────────────────────────

ALL_SERVICES: list[str] = ["postgres", "worker", "backend", "frontend", "nginx"]

PROFILES: dict[str, list[str] | None] = {
    "Full (all services)":       ["postgres", "worker", "backend", "frontend", "nginx"],
    "Worker only":               ["worker"],
    "DB only":                   ["postgres"],
    "Everything without worker": ["postgres", "backend", "frontend", "nginx"],
    "Custom":                    None,
}


def pick_services() -> list[str]:
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

    success(f"Profile: {name}")
    print(  f"  Services: {', '.join(svcs)}")
    return svcs


# ── Postgres readiness ────────────────────────────────────────────────────────

def wait_for_local_postgres(pg_user: str, retries: int = 20, delay: float = 3.0) -> bool:
    """Poll the local postgres container until it accepts connections."""
    print("\n  Waiting for postgres", end="", flush=True)
    for _ in range(retries):
        r = subprocess.run(
            ["docker", "compose", "exec", "postgres", "pg_isready", "-U", pg_user],
            capture_output=True,
        )
        if r.returncode == 0:
            print(_c("  ready!", GREEN))
            return True
        print(".", end="", flush=True)
        time.sleep(delay)
    print()
    return False


def wait_for_remote_postgres(db: DatabaseConfig, retries: int = 10, delay: float = 3.0) -> bool:
    """Poll a remote postgres until it accepts connections (uses pg_isready on host)."""
    if shutil.which("pg_isready") is None:
        warn("pg_isready not found on host — skipping remote DB readiness check.")
        return True
    print("\n  Waiting for remote postgres", end="", flush=True)
    for _ in range(retries):
        r = subprocess.run(
            ["pg_isready", "-h", db.host, "-p", db.port, "-U", db.user],
            capture_output=True,
        )
        if r.returncode == 0:
            print(_c("  ready!", GREEN))
            return True
        print(".", end="", flush=True)
        time.sleep(delay)
    print()
    return False


# ── Diesel migrations ─────────────────────────────────────────────────────────

def git_pull() -> None:
    """Pull latest commits, discarding any local schema.rs changes first.

    diesel migration run regenerates schema.rs as a side-effect, so the file
    is always dirty after a migration.  Trying to pull with a dirty schema.rs
    causes a merge conflict every time.  We discard it unconditionally because:
      - it is auto-generated (not hand-edited)
      - the correct version will be regenerated by the migration that follows
    """
    if not Path(".git").is_dir():
        warn("Not a git repository — skipping git pull.")
        return
    Checks.git()

    schema = DB_SCHEMA / "src" / "schema.rs"
    if schema.exists():
        r = subprocess.run(
            ["git", "status", "--porcelain", str(schema)],
            capture_output=True, text=True,
        )
        if r.stdout.strip():
            warn(f"Discarding local changes to {schema} (auto-generated by diesel).")
            run(["git", "checkout", "--", str(schema)])

    run(["git", "pull"])


def run_diesel_migrations(cfg: AppConfig) -> None:
    header("DIESEL MIGRATIONS")

    if not DB_SCHEMA.is_dir():
        warn(f"'{DB_SCHEMA}/' directory not found — skipping migrations.")
        return

    Checks.diesel()

    git_pull()

    url = cfg.db.migration_url()
    run(
        ["diesel", "migration", "run"],
        extra_env={"DATABASE_URL": url},
        cwd=str(DB_SCHEMA),
    )
    success("Diesel migrations applied.")


# ── Prompt helpers ────────────────────────────────────────────────────────────

def prompt_db(existing: DatabaseConfig, remote: bool) -> DatabaseConfig:
    user     = ask("POSTGRES_USER",     existing.user)
    password = ask("POSTGRES_PASSWORD", existing.password)
    db       = ask("POSTGRES_DB",       existing.db)
    port     = existing.port
    host     = ""

    if remote:
        warn("No local postgres selected. Enter remote DB connection details.")
        host = ask("DB_HOST (remote host)", existing.host)
        port = ask("DB_PORT (remote port)", existing.port)

    return DatabaseConfig(user=user, password=password, db=db, host=host, port=port)


def prompt_worker(existing: WorkerConfig) -> WorkerConfig:
    return WorkerConfig(
        threads                = ask("THREADS",                existing.threads),
        search_module          = ask("SEARCH_MODULE",          existing.search_module),
        update_module          = ask("UPDATE_MODULE",          existing.update_module),
        update_with_connection = ask("UPDATE_WITH_CONNECTION", existing.update_with_connection),
        rust_log               = ask("RUST_LOG (info/debug)",  existing.rust_log),
        only_update_spoofable  = ask("ONLY_UPDATE_SPOOFABLE",  existing.only_update_spoofable),
    )


# ── DATABASE_URL injection into .env ─────────────────────────────────────────
#
# docker-compose reads .env and resolves ${DATABASE_URL} inside service
# definitions, so we write it there rather than baking it into the YAML.

def inject_database_url(cfg: AppConfig) -> None:
    """Add DATABASE_URL to the .env so compose can resolve it in services."""
    raw = load_raw_env()
    local_container = POSTGRES_CONTAINER
    raw["DATABASE_URL"] = cfg.db.service_url(local_container)
    lines = [f"{k}={v}" for k, v in raw.items()]
    ENV_FILE.write_text("\n".join(lines) + "\n", encoding="utf-8")


# ── Mode: Install ─────────────────────────────────────────────────────────────

class InstallMode:
    def run(self) -> None:
        header("INSTALL")
        Checks.docker()

        # Warn if compose already exists to avoid clobbering production overrides
        if COMPOSE_FILE.exists():
            warn(
                "docker-compose.yml already exists.\n"
                "  If you have production overrides (Let's Encrypt ports, etc.) they will be lost."
            )
            choice = menu("How do you want to proceed?", [
                "Overwrite docker-compose.yml",
                "Keep existing docker-compose.yml (only update .env)",
                "Abort",
            ])
            if choice == 2:
                print("  Aborted.")
                sys.exit(0)
            keep_compose = (choice == 1)
        else:
            keep_compose = False

        services       = pick_services()
        local_postgres = "postgres" in services
        needs_backend  = "backend" in services
        needs_db       = any(s in services for s in ("worker", "backend"))

        raw     = load_raw_env()
        existing = AppConfig.from_env(raw)

        print("\n  Configure environment variables:")

        db = prompt_db(existing.db, remote=(not local_postgres and needs_db))
        backend_password = ask("BACKEND_PASSWORD", existing.backend_password) if needs_backend else existing.backend_password
        worker = prompt_worker(existing.worker) if "worker" in services else existing.worker

        cfg = AppConfig(db=db, backend_password=backend_password, worker=worker, services=services)
        save_env(cfg)
        inject_database_url(cfg)

        if not keep_compose:
            ComposeBuilder(services).save()

        run(["docker", "compose", "pull"])

        if needs_db and ask_yes("\nRun diesel migrations?", default=True):
            self._handle_migrations(cfg, local_postgres)

        if ask_yes("\nRun docker compose up?", default=True):
            run(["docker", "compose", "up", "-d"])

        success("Installation complete.")

    def _handle_migrations(self, cfg: AppConfig, local_postgres: bool) -> None:
        if local_postgres:
            run(["docker", "compose", "up", "postgres", "-d"])
            if wait_for_local_postgres(cfg.db.user):
                run_diesel_migrations(cfg)
            else:
                warn("Postgres did not become ready in time — skipping migrations.")
        else:
            if wait_for_remote_postgres(cfg.db):
                run_diesel_migrations(cfg)
            else:
                warn("Remote postgres did not become ready in time — skipping migrations.")


# ── Mode: Update ──────────────────────────────────────────────────────────────

class UpdateMode:
    def run(self) -> None:
        header("UPDATE")
        Checks.docker()

        options = [
            "Pull latest images & restart all services",
            "Pull latest images & restart one service",
            "Run diesel migrations (git pull first)",
            "Start everything",
            "Stop everything",
        ]
        choice = menu("What would you like to update?", options)

        dispatch = {
            0: self._update_all,
            1: self._update_one,
            2: self._migrate,
            3: self._start,
            4: self._stop,
        }
        dispatch[choice]()

    def _update_all(self) -> None:
        run(["docker", "compose", "down"])
        run(["docker", "compose", "pull"])
        run(["docker", "compose", "up", "-d"])
        success("All services updated.")

    def _update_one(self) -> None:
        svc = ask("Service name (worker / backend / frontend / nginx)")
        run(["docker", "compose", "down", svc])
        run(["docker", "compose", "pull", svc])
        run(["docker", "compose", "up", svc, "-d"])
        success(f"'{svc}' updated.")

    def _migrate(self) -> None:
        cfg = AppConfig.from_env(load_raw_env())
        local_postgres = "postgres" in ComposeBuilder.active_services()
        if local_postgres:
            if not wait_for_local_postgres(cfg.db.user):
                warn("Postgres not ready — aborting migrations.")
                return
        else:
            if not wait_for_remote_postgres(cfg.db):
                warn("Remote postgres not ready — aborting migrations.")
                return
        run_diesel_migrations(cfg)

    def _start(self) -> None:
        run(["docker", "compose", "up", "-d"])

    def _stop(self) -> None:
        run(["docker", "compose", "down"])


# ── Mode: Settings ────────────────────────────────────────────────────────────

class SettingsMode:
    # Each entry: (env_key, prompt_label, always_show)
    FIELDS: list[tuple[str, str]] = [
        ("POSTGRES_USER",           "PostgreSQL user"),
        ("POSTGRES_PASSWORD",       "PostgreSQL password"),
        ("POSTGRES_DB",             "PostgreSQL database name"),
        ("POSTGRES_PORT",           "PostgreSQL port"),
        ("DB_HOST",                 "Remote DB host (leave blank for local Docker)"),
        ("BACKEND_PASSWORD",        "Backend API password"),
        ("THREADS",                 "Worker threads"),
        ("SEARCH_MODULE",           "Search module enabled (true/false)"),
        ("UPDATE_MODULE",           "Update module enabled (true/false)"),
        ("UPDATE_WITH_CONNECTION",  "Update with connection (true/false)"),
        ("ONLY_UPDATE_SPOOFABLE",   "Only update spoofable servers (true/false)"),
        ("RUST_LOG",                "Worker log level (info/debug)"),
    ]

    def run(self) -> None:
        header("CHANGE SETTINGS")

        raw = load_raw_env()
        if not raw:
            warn(".env not found or empty — starting from scratch.")

        print("  Press Enter to keep the current value.\n")

        changed = False
        for key, label in self.FIELDS:
            current = raw.get(key, "")
            new_val = ask(label, current)
            if new_val != current:
                if new_val == "":
                    raw.pop(key, None)
                else:
                    raw[key] = new_val
                changed = True

        if not changed:
            print("\n  No changes made.")
            return

        cfg = AppConfig.from_env(raw)
        save_env(cfg)
        inject_database_url(cfg)

        # Regenerate compose only if it already exists and user agrees
        if COMPOSE_FILE.exists() and ask_yes("Regenerate docker-compose.yml to reflect new settings?", default=False):
            services = ComposeBuilder.active_services()
            if services:
                ComposeBuilder(services).save()
            else:
                warn("Could not determine active services — compose file not updated.")

        if ask_yes("\nRestart services to apply new settings?", default=True):
            Checks.docker()
            run(["docker", "compose", "up", "-d"])
            success("Services restarted with new settings.")


# ── Main ──────────────────────────────────────────────────────────────────────

def main() -> None:
    print(f"\n{BOLD}{CYAN}  mine_search  —  Setup Script{RESET}")

    modes: dict[str, type] = {
        "install":  InstallMode,
        "i":        InstallMode,
        "update":   UpdateMode,
        "u":        UpdateMode,
        "settings": SettingsMode,
        "s":        SettingsMode,
    }

    if len(sys.argv) > 1:
        arg = sys.argv[1].lower()
        cls = modes.get(arg)
        if cls:
            cls().run()
            return
        error(f"Unknown argument '{arg}'. Valid: install (i), update (u), settings (s)")
        sys.exit(1)

    choice = menu("Select mode:", [
        "Install  — fresh setup, choose which modules to deploy",
        "Update   — pull latest images, run migrations, rebuild",
        "Settings — change .env values and restart",
        "Exit",
    ])
    [InstallMode, UpdateMode, SettingsMode, type("_Exit", (), {"run": staticmethod(lambda: print("  Bye!\n"))})][choice]().run()


if __name__ == "__main__":
    main()