//! In-process pub/sub for "a server's row changed" notifications. When a worker
//! result is persisted, the backend broadcasts the affected `server_id`; the
//! `StreamServerInfo` RPC subscribes and re-pushes fresh `ServerInfo` to any
//! frontend currently viewing that server's detail page.

use tokio::sync::broadcast;

/// Broadcast capacity. Lagged receivers drop the oldest ids — harmless here,
/// since a missed id just means a slightly stale view that the next event (or
/// the receiver's reload-on-lag) corrects.
const CAPACITY: usize = 256;

pub struct ServerEvents {
    tx: broadcast::Sender<i32>,
}

impl Default for ServerEvents {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(CAPACITY);
        Self { tx }
    }
}

impl ServerEvents {
    /// Signals that `server_id`'s row changed. Ignores the "no subscribers" case.
    pub fn notify(&self, server_id: i32) {
        let _ = self.tx.send(server_id);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<i32> {
        self.tx.subscribe()
    }
}
