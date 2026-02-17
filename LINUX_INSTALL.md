# Linux Installation Guide

Follow these steps to set up MineSearch on a Linux system.

## Prerequisites

- [Docker](https://docs.docker.com/engine/install/) installed and running
- [Docker Compose](https://docs.docker.com/compose/install/) (usually included with Docker)
- [Git](https://git-scm.com/) (optional, to clone the repository)
- `curl` (for installing Diesel CLI)

## Step 1: Clone the Repository

    git clone https://github.com/kauri-off/mine_search
    cd mine_search

## Step 2: Install Diesel CLI

Run the official installer:

    curl --proto '=https' --tlsv1.2 -LsSf https://github.com/diesel-rs/diesel/releases/latest/download/diesel_cli-installer.sh | sh

Verify the installation:

    ~/.cargo/bin/diesel --version

Make sure the `diesel` command is in your `PATH` (the installer usually adds it automatically).

## Step 3: Start PostgreSQL

Start the PostgreSQL container using Docker Compose:

    docker compose up postgres -d

## Step 4: Set Up Environment Variables

Create a `.env` file in the project root as described in the main [README](README.md#environment-variables).
Then export the variables and set the `DATABASE_URL`:

    export $(grep -v '^#' .env | xargs)
    export DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@127.0.0.1:5432/${POSTGRES_DB}"

## Step 5: Run Database Migrations

Navigate to the `db_schema` directory and apply the migrations:

    cd db_schema
    diesel migration run

## Next Steps

You have now successfully set up the MineSearch environment on Linux.
Return to the main [README](README.md#running-the-application) to learn how to run the application.
