# MineSearch

**mine_search** is a Minecraft server search engine. It allows you to search for Minecraft servers using a PostgreSQL database. The application is designed to be lightweight, flexible, and easy to deploy.

## Features

- **Search Minecraft servers**: Quickly find information about Minecraft servers from the database.
- **PostgreSQL database**: All server data is stored in a robust PostgreSQL database.
- **Docker support**: Easily run the application in a Docker container.
- **Environment configuration**: Specify the connection details to the PostgreSQL database in a `.env` file.

## Getting Started

### Prerequisites

- [Docker](https://www.docker.com/) installed on your system
- PostgreSQL database with server data
- [Diesel CLI](https://diesel.rs/) for managing the database schema

### Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/kauri-off/mine_search
   cd mine_search
   ```

2. Create a `.env` file in the root directory and specify the connection details to your PostgreSQL database:

   ```env
   DATABASE_URL=postgres://username:password@localhost:5432/mine_search_db
   ```

   If you are running the application using Docker, specify the connection details for the PostgreSQL container, for example:

   ```env
   POSTGRES_USER=postgres
   POSTGRES_PASSWORD=password
   POSTGRES_DB=mine_search_db
   ```

3. Set up the database using Diesel:
   ```bash
   diesel setup
   ```

### Running the Application

#### Using Docker

1. Build and run the application in a Docker container:

   ```bash
   source .env
   docker compose up -d --build
   ```

2. The application will be accessible based on the configuration in the `docker-compose.yml` file.

#### Native Execution

1. Ensure you have Rust and Cargo installed on your system.
2. Run the application directly:
   ```bash
   source .env
   cargo run
   ```

## Usage

- For now, it's just a raw database, but later I will add a web panel.

## Environment Variables

| Variable            | Description                             |
| ------------------- | --------------------------------------- |
| `THREADS`           | Number of threads searching for servers |
| `DATABASE_URL`      | PostgreSQL connection string            |
| `POSTGRES_USER`     | PostgreSQL username                     |
| `POSTGRES_PASSWORD` | PostgreSQL password                     |
| `POSTGRES_DB`       | PostgreSQL database name                |

## License

This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests to improve the project.

## Acknowledgements

- [PostgreSQL](https://www.postgresql.org/)
- [Diesel](https://diesel.rs/)
- [Docker](https://www.docker.com/)
