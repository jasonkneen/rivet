# SQLite VFS (`@rivetkit/sqlite-vfs`)

## How It Works
- SQLite issues byte-range reads/writes; VFS translates to chunked KV operations
- `CHUNK_SIZE = 4096` — each chunk is one KV key
- `xWrite`: computes touched chunks, read-modify-write for partial updates, `putBatch`
- `xRead`: fetches chunk range, copies bytes, zero-fills gaps
- Metadata (file size) stored alongside chunks via `metaKey`

## Single-Writer Model
- Actors are single-writer, so `xLock`/`xUnlock` are no-ops
- No need for WAL (its benefit is concurrent readers/writer)
- Double mutex exists: `db/mod.ts` + `vfs.ts` — redundant under single-writer

## Current Journal/WAL Status
- Actor KV path: DELETE journal mode (SQLite default), no WAL
- File-system driver: uses WAL (standard WAL, not WAL2)
- WAL not recommended for KV-backed VFS due to checkpoint traffic on high-latency KV

## Caching
- SQLite has its own page cache; VFS-level chunk cache would mostly duplicate it
- VFS cache only helps if KV RTT is very high and pages churn — treat as benchmark-driven, not default

## Pending TODOs
- Measure `xAccess` KV round-trip overhead during DB open
- Benchmark `journal_mode=PERSIST` + `journal_size_limit` (fewer KV deletes per txn)
- Fast-path delete-on-close: reuse in-memory `file.size` instead of extra `metaKey` read

## Decisions Made
- Do NOT defer metadata writes to `xSync`/`xClose` — crash risk outweighs minimal gain (metadata already batched with chunk data in `putBatch`)
- Do NOT enable `journal_mode=MEMORY`, `journal_mode=OFF`, or `synchronous=OFF`
- `journal_mode=PERSIST` is safe to switch to later (no migration needed)

## Future Work
- **PITR / fork**: implement at KV layer (immutable chunk versions, manifests, branch heads, GC) with SQLite layer providing snapshot boundary coordination
- **Remove double mutex** once profiled
