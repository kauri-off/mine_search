## Architecture

The frontend⇆backend API and the backend⇆worker control plane both run over
**gRPC** (tonic). The backend is the gRPC server and the sole DB writer; workers
dial in, register, stream scan results + heartbeats, and receive commands.

- `proto/` — shared `.proto` files (`api.proto`, `worker.proto`) + generated Rust
  (tonic, via `protox` so no `protoc` binary is needed).
- `backend/` — tonic server: `ApiService` (frontend, served as gRPC-web via
  tonic-web) + `WorkerControlService` (workers). In-memory `WorkerRegistry`.
- `worker/` — Minecraft scanner. **Default feature `grpc`** (streams to backend).
  Optional **`diesel`** feature (default OFF) writes to PostgreSQL directly.
- `frontend/` — React + Connect-ES gRPC-web client.

## Types: proto → TS/Rust (gRPC)

Wire types come from `proto/proto/*.proto`. The old ts-rs export workflow is
**retired**; `frontend/src/types/*` are now hand-maintained UI view-models that
the adapter in `frontend/src/api/client.ts` maps protobuf messages to.

### Workflow when changing the API

1. Edit `proto/proto/api.proto` (or `worker.proto`).
2. Rust regenerates on build (`cargo build`); update the service impls.
3. Regenerate the frontend client: `cd frontend && npx buf generate` (outputs to
   `src/gen/`). Update the adapter/view-models if shapes changed.

## Type checking for frontend

Always use `npx tsc -b --noEmit` (not `npx tsc --noEmit`)

## Building (Windows local dev)

`openssl-sys` is vendored for Docker/Linux. On this Windows box the vendored
build fails (broken msys perl), so use the prebuilt system OpenSSL:

```
export OPENSSL_NO_VENDOR=1 \
  OPENSSL_DIR="C:\\Program Files\\OpenSSL-Win64" \
  OPENSSL_LIB_DIR="C:\\Program Files\\OpenSSL-Win64\\lib\\VC\\x64\\MD" \
  OPENSSL_INCLUDE_DIR="C:\\Program Files\\OpenSSL-Win64\\include"
cargo check --workspace
```

Worker feature builds: `cargo check -p worker` (grpc, default) and
`cargo check -p worker --no-default-features --features diesel`.

## Database schema

`db_schema/src/schema.rs` (diesel migrations in `db_schema/migrations/`).
