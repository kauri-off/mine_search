# MineSearch

**mine_search** is a Minecraft server search engine. It allows you to search for Minecraft servers using a SQLite3 database. The application is designed to be lightweight, flexible, and easy to deploy.

## Features

- **Search Minecraft servers**: Quickly find information about Minecraft servers from the database.
- **SQLite3 database**: All server data is stored in a lightweight SQLite3 database.
- **Docker support**: Easily run the application in a Docker container.
- **Environment configuration**: Specify the path to the SQLite3 database in a `.env` file.

## Getting Started

### Prerequisites

- [Docker](https://www.docker.com/) installed on your system
- SQLite3 database file with server data
- [Diesel CLI](https://diesel.rs/) for managing the database schema

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/kauri-off/mine_search
   cd mine_search
   ```

2. Create a `.env` file in the root directory and specify the path to your SQLite3 database file:
   ```env
   DATABASE_URL=/path/to/your/database.db
   ```

3. Set up the database using Diesel:
   ```bash
   diesel setup
   ```

4. Run app with docker:
   ```bash
   docker compose up -d --build
   ```

5. Or run native:
   ```bash
   cargo run
   ```

## Usage

- For now, it's just a raw database, but later I will add a web panel.

## Environment Variables

| Variable       | Description                         |
|----------------|-------------------------------------|
| `WORKERS` | Number of threads searching for a servers |

## License

This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests to improve the project.

## Acknowledgements

- [SQLite](https://www.sqlite.org/)
- [Diesel](https://diesel.rs/)
- [Docker](https://www.docker.com/)

