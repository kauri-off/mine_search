# Windows Installation Guide

Follow these steps to set up MineSearch on a Windows system.

## Prerequisites

- [Docker Desktop](https://www.docker.com/products/docker-desktop/) installed and running (includes Docker Compose)
- [Git for Windows](https://git-scm.com/download/win) (optional, to clone the repository)
- [Rust](https://rustup.rs/) (recommended for installing Diesel CLI) – or download the Diesel binary manually

## Step 1: Clone the Repository

Open PowerShell or Command Prompt and run:

```powershell
git clone https://github.com/kauri-off/mine_search
cd mine_search
```

## Step 2: Install Diesel CLI

You have two options:

### Option A: Install via Cargo (requires Rust and Perl)

If you have Rust and Perl installed, run:

```bash
cargo install diesel_cli --no-default-features --features postgres-bundled
```

### Option B: Download Pre‑built Binary

```powershell
Set-ExecutionPolicy RemoteSigned -scope CurrentUser

irm https://github.com/diesel-rs/diesel/releases/download/v2.3.5/diesel_cli-installer.ps1 | iex
```

Verify the installation:

```powershell
diesel --version
```

## Step 3: Set Up Environment Variables

Create a `.env` file in the project root with the following content:

```env
POSTGRES_USER=user
POSTGRES_PASSWORD=CHANGE_THIS
POSTGRES_DB=mine_search_db

BACKEND_PASSWORD=CHANGE_THIS

THREADS=500
ONLY_UPDATE=false
UPDATE_WITH_CONNECTION=false
```

Then open **PowerShell** and set the `DATABASE_URL` by reading the values from your `.env` file:

```powershell
$env:DATABASE_URL = "postgresql://$((Get-Content .env | ConvertFrom-StringData).POSTGRES_USER):$((Get-Content .env | ConvertFrom-StringData).POSTGRES_PASSWORD)@localhost:5432/$((Get-Content .env | ConvertFrom-StringData).POSTGRES_DB)"
```

(If you are using Command Prompt, you will need to set the variables manually or use a different approach; PowerShell is recommended.)

## Step 4: Start PostgreSQL

Start the PostgreSQL container using Docker Compose:

```powershell
docker compose up postgres -d
```

## Step 5: Run Database Migrations

Navigate to the `db_schema` directory and apply the migrations:

```powershell
cd db_schema
diesel migration run
```

## Database Migration After Updates

When you update the repository (e.g., via `git pull`) and the update includes changes to the database schema, you need to run database migrations to apply those changes.

### **Step 1: Ensure PostgreSQL is Running**

Make sure the PostgreSQL container is running:

```powershell
docker compose up postgres -d
```

### **Step 2: Set Environment Variables**

Ensure your environment variables are set. In PowerShell, set the `DATABASE_URL`:

```powershell
$env:DATABASE_URL = "postgresql://$((Get-Content .env | ConvertFrom-StringData).POSTGRES_USER):$((Get-Content .env | ConvertFrom-StringData).POSTGRES_PASSWORD)@localhost:5432/$((Get-Content .env | ConvertFrom-StringData).POSTGRES_DB)"
```

### **Step 3: Run Migrations**

Navigate to the `db_schema` directory and run the migrations:

```powershell
cd db_schema
diesel migration run
```

### **Step 4: Restart Services**

After migrations are applied, restart the application services:

```powershell
docker compose up -d --build
```

## Next Steps

You have now successfully set up the MineSearch environment on Windows.
Return to the main [README](README.md#running-the-application) to learn how to run the application.
