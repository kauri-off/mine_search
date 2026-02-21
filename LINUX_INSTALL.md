# Linux Installation Guide

Follow these steps to set up MineSearch on a Linux system.

## Prerequisites

- [Docker](https://docs.docker.com/engine/install/) installed and running
- [Docker Compose](https://docs.docker.com/compose/install/) (usually included with Docker)
- [Git](https://git-scm.com/) (optional, to clone the repository)
- `curl` (for installing Diesel CLI)

## Step 1: Clone the Repository

```bash
git clone https://github.com/kauri-off/mine_search
cd mine_search
```

## Step 2: Install Diesel CLI

Run the official installer:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/diesel-rs/diesel/releases/latest/download/diesel_cli-installer.sh | sh
```

Verify the installation:

```bash
~/.cargo/bin/diesel --version
```

Make sure the `diesel` command is in your `PATH` (the installer usually adds it automatically).

## Step 3: Set Up Environment Variables

Create a `.env` file in the project root with the following content:

```env
POSTGRES_USER=user
POSTGRES_PASSWORD=CHANGE_THIS
POSTGRES_DB=mine_search_db

BACKEND_PASSWORD=CHANGE_THIS

THREADS=500
SEARCH_MODULE=true
UPDATE_MODULE=true
UPDATE_WITH_CONNECTION=false
```

Then export the variables and set the `DATABASE_URL`:

```bash
export $(grep -v '^#' .env | xargs)
export DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@127.0.0.1:5432/${POSTGRES_DB}"
```

## Step 4: Start PostgreSQL

Start the PostgreSQL container using Docker Compose:

```bash
docker compose up postgres -d
```

## Step 5: Run Database Migrations

Navigate to the `db_schema` directory and apply the migrations:

```bash
cd db_schema
diesel migration run
```

## Database Migration After Updates

When you update the repository (e.g., via `git pull`) and the update includes changes to the database schema, you need to run database migrations to apply those changes.

### **Step 1: Ensure PostgreSQL is Running**

Make sure the PostgreSQL container is running:

```bash
docker compose up postgres -d
```

### **Step 2: Set Environment Variables**

Ensure your environment variables are set. Export them in your terminal:

```bash
export $(grep -v '^#' .env | xargs)
export DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@127.0.0.1:5432/${POSTGRES_DB}"
```

### **Step 3: Run Migrations**

Navigate to the `db_schema` directory and run the migrations:

```bash
cd db_schema
diesel migration run
```

### **Step 4: Restart Services**

After migrations are applied, restart the application services:

```bash
docker compose up -d --build
```

## Next Steps

You have now successfully set up the MineSearch environment on Linux.
Return to the main [README](README.md#running-the-application) to learn how to run the application.
