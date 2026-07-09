//! Durable, bounded outbox for scan results awaiting backend acknowledgement.
//!
//! The worker streams `ScanResult`s to the backend, but the gRPC link can drop
//! (or the backend's DB can be down) at any moment, and a session's in-memory
//! send channel is discarded on reconnect. To avoid losing discovered servers,
//! every result is recorded here — keyed by its `result_id` — *before* it is put
//! on the wire, and only removed once the backend acks it (`Ack{result_id}`).
//! On (re)connect the worker replays everything still pending; a periodic sweep
//! re-sends anything that has gone too long without an ack (covers "link up but
//! backend can't persist"). Delivery is therefore at-least-once; the backend
//! deduplicates by `result_id`, so replays never double-write.
//!
//! Persistence is a simple append-only binary log next to the worker config:
//! a `PUT` record per result and an `ACK` record when it is acknowledged. The
//! log is replayed and compacted on startup, and compacted again once it has
//! grown well past the live entry count.

use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{self, BufReader, Read, Write},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use prost::Message;
use proto::worker::WorkerMessage;
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Cap on un-acked entries; the oldest is dropped past this to bound disk use.
const MAX_ENTRIES: usize = 10_000;
/// Entries un-acked longer than this are given up on (the backend has had ample
/// time, and the same server will be rediscovered). Kept in sync with the
/// backend's `processed_results` retention so a replay is always deduplicated.
const MAX_AGE: Duration = Duration::from_secs(24 * 3600);
/// Compact the log once it holds this many records beyond the live set.
const COMPACT_THRESHOLD: u64 = 20_000;

const OP_PUT: u8 = 1;
const OP_ACK: u8 = 2;

struct Entry {
    msg: WorkerMessage,
    /// When the result was first created (persisted; drives age-out).
    created: SystemTime,
    /// When it was last put on the wire (in-memory only; drives re-send).
    last_sent: SystemTime,
}

struct State {
    entries: HashMap<String, Entry>,
    file: File,
    appends_since_compact: u64,
}

pub struct Outbox {
    path: PathBuf,
    state: Mutex<State>,
}

impl Outbox {
    /// Opens (or creates) the outbox at `path`, replaying and compacting any
    /// existing log so memory and disk agree.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let entries = load_entries(&path);
        let count = entries.len();
        let file = compact_to_disk(&path, &entries)?;
        if count > 0 {
            info!("outbox: recovered {count} pending scan result(s) from {}", path.display());
        }
        Ok(Self {
            path,
            state: Mutex::new(State {
                entries,
                file,
                appends_since_compact: 0,
            }),
        })
    }

    /// Records a result before it is sent. Evicts the oldest entry if full.
    pub async fn add(&self, id: &str, msg: &WorkerMessage) {
        let mut st = self.state.lock().await;
        let now = SystemTime::now();

        if st.entries.len() >= MAX_ENTRIES && !st.entries.contains_key(id) {
            if let Some(oldest) = oldest_id(&st.entries) {
                st.entries.remove(&oldest);
                let _ = append(&mut st.file, OP_ACK, 0, &oldest, &[]);
                warn!("outbox: full ({MAX_ENTRIES}); dropped oldest un-acked result");
            }
        }

        let bytes = msg.encode_to_vec();
        if append(&mut st.file, OP_PUT, unix_secs(now), id, &bytes).is_err() {
            warn!("outbox: failed to persist result {id}");
        }
        st.appends_since_compact += 1;
        st.entries.insert(
            id.to_string(),
            Entry { msg: msg.clone(), created: now, last_sent: now },
        );

        maybe_compact(&mut st, &self.path);
    }

    /// Marks a result acknowledged and removes it.
    pub async fn ack(&self, id: &str) {
        let mut st = self.state.lock().await;
        if st.entries.remove(id).is_some() {
            let _ = append(&mut st.file, OP_ACK, 0, id, &[]);
            st.appends_since_compact += 1;
            maybe_compact(&mut st, &self.path);
        }
    }

    /// Returns results to (re-)send. With `resend_after = None` returns every
    /// pending result (use on reconnect); otherwise only those not sent within
    /// `resend_after` (the periodic sweep). Ages out entries past [`MAX_AGE`]
    /// and stamps the returned ones as just-sent.
    pub async fn collect(&self, resend_after: Option<Duration>) -> Vec<WorkerMessage> {
        let mut st = self.state.lock().await;
        let now = SystemTime::now();

        // Age out abandoned entries.
        let expired: Vec<String> = st
            .entries
            .iter()
            .filter(|(_, e)| now.duration_since(e.created).unwrap_or_default() > MAX_AGE)
            .map(|(id, _)| id.clone())
            .collect();
        for id in expired {
            st.entries.remove(&id);
            let _ = append(&mut st.file, OP_ACK, 0, &id, &[]);
            st.appends_since_compact += 1;
            warn!("outbox: gave up on result {id} (un-acked for > {}h)", MAX_AGE.as_secs() / 3600);
        }

        // Select due entries and stamp them as sent.
        let due: Vec<String> = st
            .entries
            .iter()
            .filter(|(_, e)| match resend_after {
                None => true,
                Some(after) => now.duration_since(e.last_sent).unwrap_or_default() >= after,
            })
            .map(|(id, _)| id.clone())
            .collect();

        let mut out = Vec::with_capacity(due.len());
        for id in &due {
            if let Some(e) = st.entries.get_mut(id) {
                e.last_sent = now;
                out.push(e.msg.clone());
            }
        }
        maybe_compact(&mut st, &self.path);
        out
    }
}

fn unix_secs(t: SystemTime) -> u64 {
    t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn oldest_id(entries: &HashMap<String, Entry>) -> Option<String> {
    entries
        .iter()
        .min_by_key(|(_, e)| e.created)
        .map(|(id, _)| id.clone())
}

/// Rewrites the log to contain exactly the live entries (all as PUTs) when it
/// has accumulated too many superseded records.
fn maybe_compact(st: &mut State, path: &Path) {
    if st.appends_since_compact <= COMPACT_THRESHOLD {
        return;
    }
    match compact_to_disk(path, &st.entries) {
        Ok(file) => {
            st.file = file;
            st.appends_since_compact = 0;
        }
        Err(e) => warn!("outbox: compaction failed: {e}"),
    }
}

/// Writes all `entries` to a fresh temp file as PUT records, atomically renames
/// it over `path`, and returns an append handle to the result.
fn compact_to_disk(path: &Path, entries: &HashMap<String, Entry>) -> io::Result<File> {
    let tmp = path.with_extension("log.tmp");
    {
        let mut f = File::create(&tmp)?;
        for (id, e) in entries {
            let bytes = e.msg.encode_to_vec();
            append(&mut f, OP_PUT, unix_secs(e.created), id, &bytes)?;
        }
        // Force the temp file's contents to disk before the rename, so a crash
        // can never leave a renamed-but-empty log.
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    // fsync the parent directory so the rename itself survives power loss.
    // Best-effort and platform-dependent (opening a directory as a file fails on
    // Windows); the durability target is the Linux/Docker deployment.
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        if let Ok(d) = File::open(dir) {
            let _ = d.sync_all();
        }
    }
    OpenOptions::new().append(true).create(true).open(path)
}

/// Record layout: `[op:u8][created_secs:u64 LE][id_len:u32 LE][id][payload_len:u32 LE][payload]`.
fn append(file: &mut File, op: u8, created_secs: u64, id: &str, payload: &[u8]) -> io::Result<()> {
    let mut buf = Vec::with_capacity(1 + 8 + 4 + id.len() + 4 + payload.len());
    buf.push(op);
    buf.extend_from_slice(&created_secs.to_le_bytes());
    buf.extend_from_slice(&(id.len() as u32).to_le_bytes());
    buf.extend_from_slice(id.as_bytes());
    buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    buf.extend_from_slice(payload);
    file.write_all(&buf)?;
    // fsync, not just flush: `File::flush` is a no-op for `std::fs::File`, so the
    // record would only reach the OS page cache. The outbox promises durability
    // across a crash/power loss, so force the bytes to disk before returning.
    file.sync_data()
}

fn load_entries(path: &Path) -> HashMap<String, Entry> {
    let mut entries: HashMap<String, Entry> = HashMap::new();
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return entries, // no log yet
    };
    let mut r = BufReader::new(file);
    loop {
        match read_record(&mut r) {
            Ok(Some((OP_PUT, created, id, payload))) => match WorkerMessage::decode(&payload[..]) {
                Ok(msg) => {
                    entries.insert(id, Entry { msg, created, last_sent: UNIX_EPOCH });
                }
                Err(e) => warn!("outbox: skipping undecodable record {id}: {e}"),
            },
            Ok(Some((OP_ACK, _, id, _))) => {
                entries.remove(&id);
            }
            Ok(Some((op, _, _, _))) => warn!("outbox: unknown record op {op}; skipping"),
            Ok(None) => break, // clean EOF
            Err(e) => {
                warn!("outbox: log truncated/corrupt, stopping replay: {e}");
                break;
            }
        }
    }
    entries
}

/// Reads one record. Returns `Ok(None)` on a clean EOF at a record boundary.
fn read_record<R: Read>(r: &mut R) -> io::Result<Option<(u8, SystemTime, String, Vec<u8>)>> {
    let mut op = [0u8; 1];
    match r.read_exact(&mut op) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }

    let mut u64buf = [0u8; 8];
    r.read_exact(&mut u64buf)?;
    let created = UNIX_EPOCH + Duration::from_secs(u64::from_le_bytes(u64buf));

    let id = read_len_prefixed(r)?;
    let id = String::from_utf8(id).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let payload = read_len_prefixed(r)?;

    Ok(Some((op[0], created, id, payload)))
}

fn read_len_prefixed<R: Read>(r: &mut R) -> io::Result<Vec<u8>> {
    let mut lenbuf = [0u8; 4];
    r.read_exact(&mut lenbuf)?;
    let len = u32::from_le_bytes(lenbuf) as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    Ok(buf)
}
