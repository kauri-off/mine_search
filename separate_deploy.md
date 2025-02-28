# MineSearch Deployment Guide: Running Worker on a Separate Server and Securing Database Access

**WARNING: RUNNING THE WORKER ON A SEPARATE SERVER MAY RESULT IN YOUR DATABASE SERVER BEING EXPOSED. IMPROPER CONFIGURATION COULD LEAD TO UNAUTHORIZED ACCESS OR BLOCKING. FOLLOW THE INSTRUCTIONS CAREFULLY!**

This guide explains how to:

- Modify your docker-compose configuration to run the **worker** container on a separate server.
- Set the correct database address in the worker container.
- Open the database port on the main server.
- Configure iptables so that only a specific (worker) server can access the database.

---

## 1. Modifying docker-compose for Remote Worker

In your current **docker-compose.yml**, the worker and backend services reference the database by hostname `postgres-container`. When running the worker on a separate server, the worker must connect to the main server using its public IP (or domain). For example, if your main server’s public IP is `192.168.0.0`, update the `DATABASE_URL` in the **worker** service.

### Modified snippet for remote worker:

```yaml
services:
  worker:
    build:
      context: .
      dockerfile: worker.Dockerfile
    # image: ghcr.io/kauri-off/mine_search/worker:latest
    environment:
      THREADS: ${THREADS}
      # Replace 192.168.0.0 with the public IP or domain of your main server
      DATABASE_URL: postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@192.168.0.0/${POSTGRES_DB}
      ONLY_UPDATE: ${ONLY_UPDATE}
    restart: unless-stopped
```

> Note: On the remote worker server, you only need to run the worker container (and any other relevant services). The postgres, backend, and frontend services remain on the main server.

## 2. Opening the Database Port on the Main Server

The binding to 127.0.0.1 restricts access to local connections only. To allow connections from your worker server, modify the ports mapping so PostgreSQL listens on all interfaces:

```yaml
# Main server
ports:
  - "5432:5432/tcp"
```

Warning: Opening the port to the public network can expose your database. You must secure access with firewall rules.

## 3. Configuring iptables to Restrict Database Access

To ensure that only your worker server can access the PostgreSQL port (5432) on the main server, configure iptables as follows (change WORKER_IP to IP of your worker server):

1. Allow connections from the worker server:

```bash
sudo iptables -A INPUT -p tcp -s WORKER_IP --dport 5432 -j ACCEPT
```

2. Allow connections from localhost (diesel_cli may not work):

```bash
sudo iptables -A INPUT -p tcp -s 127.0.0.1 --dport 5432 -j ACCEPT
```

3. Drop all other incoming connections to port 5432:

```bash
sudo iptables -A INPUT -p tcp --dport 5432 -j DROP
```
