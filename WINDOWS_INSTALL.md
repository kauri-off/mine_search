# Windows Installation Guide

Follow these steps to set up MineSearch on a Windows system.

## Prerequisites

- [Docker Desktop](https://www.docker.com/products/docker-desktop/) installed and running (includes Docker Compose)
- [Git for Windows](https://git-scm.com/download/win) (optional, to clone the repository)
- [Rust](https://rustup.rs/) (recommended for installing Diesel CLI) – or download the Diesel binary manually

## Step 1: Clone the Repository

Open PowerShell or Command Prompt and run:

    git clone https://github.com/kauri-off/mine_search
    cd mine_search

## Step 2: Install Diesel CLI

You have two options:

### Option A: Install via Cargo (requires Rust and Perl)

If you have Rust and Perl installed, run:

    cargo install diesel_cli --no-default-features --features postgres-bundled

### Option B: Download Pre‑built Binary

    Set-ExecutionPolicy RemoteSigned -scope CurrentUser

    irm https://github.com/diesel-rs/diesel/releases/download/v2.3.5/diesel_cli-installer.ps1 | iex

Verify the installation:

    diesel --version

## Step 3: Start PostgreSQL

Start the PostgreSQL container using Docker Compose:

    docker compose up postgres -d

## Step 4: Set Up Environment Variables

Create a `.env` file in the project root as described in the main [README](README.md#environment-variables).
Then open **PowerShell** and set the `DATABASE_URL` by reading the values from your `.env` file:

    $env:DATABASE_URL = "postgresql://$((Get-Content .env | ConvertFrom-StringData).POSTGRES_USER):$((Get-Content .env | ConvertFrom-StringData).POSTGRES_PASSWORD)@localhost:5432/$((Get-Content .env | ConvertFrom-StringData).POSTGRES_DB)"

(If you are using Command Prompt, you will need to set the variables manually or use a different approach; PowerShell is recommended.)

## Step 5: Run Database Migrations

Navigate to the `db_schema` directory and apply the migrations:

    cd db_schema
    diesel migration run

## Next Steps

You have now successfully set up the MineSearch environment on Windows.
Return to the main [README](README.md#running-the-application) to learn how to run the application.
