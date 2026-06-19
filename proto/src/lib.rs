//! Generated gRPC bindings shared by the backend (server) and worker (client),
//! and the source of truth for the frontend's `buf` codegen.
//!
//! - [`api`] — frontend ⇆ backend service.
//! - [`worker`] — worker ⇆ backend control plane (workers dial in, register,
//!   stream scan results + heartbeats, and receive commands/config).

pub mod api {
    tonic::include_proto!("api");
}

pub mod worker {
    tonic::include_proto!("worker");
}
