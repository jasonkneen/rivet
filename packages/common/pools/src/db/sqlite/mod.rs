use dirs;
use fdb_util::{prelude::*, SERIALIZABLE};
use foundationdb::{self as fdb, options::StreamingMode, tuple::Subspace, FdbBindingError};
use uuid::Uuid;

use futures_util::{StreamExt, TryStreamExt};
use global_error::{bail, ensure, ext::AssertionError, unwrap, GlobalResult};
use sqlx::{
	migrate::MigrateDatabase,
	sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
	ConnectOptions, Sqlite,
};
use std::io::{self, Read, Write};
use std::{
	fmt::Debug,
	path::{Path, PathBuf},
	sync::{Arc, Weak},
	time::Duration,
};
use tokio::sync::{oneshot, OnceCell};
use tokio::{
	io::AsyncReadExt,
	sync::{broadcast, Mutex},
	time::Instant,
};
use tracing::Instrument as _;

use crate::{metrics, Error, FdbPool};

mod keys;

#[cfg(test)]
mod tests;

const GC_INTERVAL: Duration = Duration::from_secs(5);
const POOL_TTL: Duration = Duration::from_secs(15);
const CHUNK_SIZE: usize = 10_000; // 10 KB, not KiB, see https://apple.github.io/foundationdb/blob.html

#[derive(Debug, Clone, Copy, PartialEq)]
struct SqliteSnapshottedState {
	total_changes: i64,
	schema_version: i64,
}

/// Convention in pools is to expose `XXXXPool` type. This is not used internally, but used to
/// define pool type in other modules.
pub type SqlitePool = SqliteConnHandle;

#[derive(Debug, Clone)]
pub enum SqliteConnType {
	/// There should only be one writer per DB.
	///
	/// The file will be loaded on the the machine and accessible until evicted.
	///
	/// Make sure to evict once done with the database.
	Writer { auto_snapshot: bool },
	/// Reads the database from FDB.
	///
	/// This is not cached in any way. This does not need to be evicted.
	Reader,
}

impl SqliteConnType {
	fn is_writer(&self) -> bool {
		match self {
			SqliteConnType::Writer { .. } => true,
			SqliteConnType::Reader => false,
		}
	}

	fn is_reader(&self) -> bool {
		match self {
			SqliteConnType::Writer { .. } => false,
			SqliteConnType::Reader => true,
		}
	}
}

#[derive(Debug, thiserror::Error)]
enum SqliteFdbError {
	#[error("mismatched chunk {key_idx}, expected {chunk_count}")]
	MismatchedChunk { chunk_count: usize, key_idx: usize },
}

/// SQLite pool that's loaded on this machine.
pub struct SqliteWriterEntry {
	key_packed: KeyPacked,
	conn_type: SqliteConnType,

	/// The underlying connection.
	///
	/// This is stored as a OnceCell since we need to be able to insert the SqliteEntry in to the
	/// HashMap before the connection is initiated.
	conn_once: OnceCell<SqliteConnHandle>,

	/// Last time this pool was accessed (either by `get` or a ref was dropped, meaning the query
	/// ended)
	///
	/// Only used for writers
	last_access: Instant,

	/// State from the last snapshot
	///
	/// Only used for writers
	snapshotted_state: SqliteSnapshottedState,

	manager: SqlitePoolManagerHandleWeak,
}

impl SqliteWriterEntry {
	fn new(
		key_packed: KeyPacked,
		conn_type: SqliteConnType,
		manager: SqlitePoolManagerHandleWeak,
	) -> SqliteWriterEntry {
		SqliteWriterEntry {
			key_packed,
			conn_type,

			conn_once: OnceCell::new(),

			last_access: Instant::now(),
			snapshotted_state: SqliteSnapshottedState {
				total_changes: 0,
				schema_version: 0,
			},
			manager,
		}
	}

	#[tracing::instrument(name = "sqlite_writer_conn", skip_all)]
	async fn conn(&self) -> Result<&SqliteConnHandle, Error> {
		self.conn_once
			.get_or_try_init(|| {
				async {
					let manager = self.manager.upgrade().ok_or_else(|| {
						Error::Global(
							AssertionError::Panic {
								message: "manager is dropped".into(),
								location: global_error::location!(),
							}
							.into(),
						)
					})?;

					SqliteConn::connect(self.key_packed.clone(), self.conn_type.clone(), manager)
						.await
				}
				.instrument(tracing::info_span!("conn_connect"))
			})
			.await
	}
}

/// DB key in packed form. This is not the full FDB key, this is the DB name segment in DbDataKey.
///
/// Stored in an `Arc` since this is frequently copied around.
type KeyPacked = Arc<Vec<u8>>;

pub type SqlitePoolManagerHandle = Arc<SqlitePoolManager>;
pub type SqlitePoolManagerHandleWeak = Weak<SqlitePoolManager>;

#[derive(Clone)]
enum SqliteStorage {
	Local { path: PathBuf },
	FoundationDb,
}

pub struct SqlitePoolManager {
	/// Writer pools are kept in memory. Reader pools are one-off temporary SQLite databases.
	writer_pools: scc::HashMap<KeyPacked, SqliteWriterEntry>,
	shutdown: broadcast::Sender<()>,
	fdb: Option<FdbPool>,
	storage: SqliteStorage,
	subspace: Subspace,
}

// MARK: Public methods
impl SqlitePoolManager {
	#[tracing::instrument(name = "sqlite_pool_manager_new", skip_all)]
	pub async fn new(fdb: Option<FdbPool>) -> Result<SqlitePoolManagerHandle, Error> {
		let (shutdown, _) = broadcast::channel(1);
		let shutdown_rx = shutdown.subscribe();

		let storage = if std::env::var("_RIVET_POOL_SQLITE_FORCE_LOCAL").map_or(false, |x| x == "1")
		{
			// Use platform-specific data directory
			let path = dirs::data_local_dir()
				.ok_or_else(|| {
					Error::Io(io::Error::new(
						io::ErrorKind::NotFound,
						"Could not determine local data directory",
					))
				})?
				.join("rivet-sqlite");

			// Ensure the directory exists
			tokio::fs::create_dir_all(&path)
				.await
				.map_err(|x| Error::Global(x.into()))?;

			SqliteStorage::Local { path }
		} else {
			SqliteStorage::FoundationDb
		};

		let manager = Arc::new(SqlitePoolManager {
			writer_pools: scc::HashMap::new(),
			shutdown,
			fdb: fdb.clone(),
			storage,
			subspace: Subspace::all().subspace(&(RIVET, SQLITE)),
		});

		tokio::task::spawn(manager.clone().manager_gc_loop(shutdown_rx));

		Ok(manager)
	}

	/// Get or creates an sqlite pool for the given key
	///
	/// IMPORTANT: Do not hold a reference to this value for an extended period of time. We use
	/// this function call to determine when to GC a pool.
	#[tracing::instrument(name = "sqlite_get", skip_all)]
	pub async fn get(
		self: &Arc<Self>,
		key: impl TuplePack + Debug,
		conn_type: SqliteConnType,
	) -> Result<SqliteConnHandle, Error> {
		let start_instant = Instant::now();

		let key_packed = Arc::new(key.pack_to_vec());
		let conn_type_str = if conn_type.is_writer() {
			"writer"
		} else {
			"reader"
		};
		let mut did_insert = false;

		// Check if pool already exists
		let conn = if conn_type.is_writer() {
			// Insert entry
			let mut entry = self
				.writer_pools
				.entry_async(key_packed.clone())
				.await
				.or_insert_with(|| {
					did_insert = true;

					// NOTE: Database will be loaded lazily on first call of `.conn()`. This is not
					// for performance, this is because we need it to be a `OnceCell` in order to
					// use it with scc.
					SqliteWriterEntry::new(
						key_packed.clone(),
						conn_type.clone(),
						Arc::downgrade(self),
					)
				});

			entry.last_access = Instant::now();

			let conn = entry.conn().await?;

			conn.clone()
		} else {
			let conn = SqliteConn::connect(key_packed.clone(), conn_type, self.clone()).await?;

			conn
		};

		let dt = start_instant.elapsed().as_secs_f64();
		metrics::SQLITE_GET_POOL_DURATION
			.with_label_values(&[conn_type_str, &did_insert.to_string()])
			.observe(dt);

		Ok(conn)
	}

	/// Evicts databases from the pool and snapshots them if needed
	#[tracing::instrument(name = "sqlite_evict", skip_all)]
	pub async fn evict<T: TuplePack>(self: &Arc<Self>, keys: Vec<T>) -> Result<(), Error> {
		let keys_packed: Vec<KeyPacked> = keys
			.into_iter()
			.map(|key| Arc::new(key.pack_to_vec()))
			.collect();

		self.evict_with_key(&keys_packed)
			.await
			.map_err(Error::Global)?;

		Ok(())
	}

	/// If the databases are loaded, then force a snapshot, or wait for existing snapshot to finish
	/// writing.
	#[tracing::instrument(name = "sqlite_flush", skip_all)]
	pub async fn flush<T: TuplePack>(self: &Arc<Self>, keys: Vec<T>) -> Result<(), Error> {
		let keys_packed: Vec<KeyPacked> = keys
			.into_iter()
			.map(|key| Arc::new(key.pack_to_vec()))
			.collect();

		self.snapshot_with_key(&keys_packed, false)
			.await
			.map_err(Error::Global)?;

		Ok(())
	}
}

// MARK: Private helpers
impl SqlitePoolManager {
	/// Inner implementation of database eviction that handles the actual removal from the pool
	#[tracing::instrument(name = "sqlite_evict_with_key", skip_all)]
	async fn evict_with_key(&self, keys_packed: &[KeyPacked]) -> GlobalResult<()> {
		if keys_packed.is_empty() {
			return Ok(());
		}

		for key_packed in keys_packed {
			tracing::debug!(key=?hex::encode(&**key_packed), "evicting sqlite database");
		}

		// Attempt to snapshot all databases in a single call
		self.snapshot_with_key(keys_packed, false).await?;

		// Remove all databases from the pools map
		// Do this after snapshotting since we don't want to remove the db if the snapshot failed.
		// If the snapshot failed, it will attempt to snapshot again on GC.
		for key_packed in keys_packed {
			self.writer_pools.remove_async(key_packed).await;
			// NOTE: The database file will be deleted when the SqliteEntry is dropped
		}

		Ok(())
	}

	/// Snapshots the current state of SQLite databases to FDB.
	///
	/// This will acquire an exclusive lock on the database to ensure consistency.
	///
	/// We can do this because we don't use WAL (since we don't need concurrent readers/writers).
	///
	/// We don't use `VACUUM FULL` because it requires significant overhead to execute frequently,
	/// which we don't need since we don't use a WAL.
	///
	/// We don't use the `.backup` command (or `sqlite3_backup_*`) because it still has some
	/// overhead.
	///
	/// Returns `true` if wrote at least one snapshot.
	#[tracing::instrument(name = "sqlite_snapshot_with_key", skip_all)]
	async fn snapshot_with_key(
		&self,
		keys_packed: &[KeyPacked],
		ensure_exists: bool,
	) -> GlobalResult<bool> {
		if keys_packed.is_empty() {
			return Ok(false);
		}

		// Only run if snapshotting required
		let SqliteStorage::FoundationDb = self.storage else {
			return Ok(false);
		};

		let start_instant = Instant::now();

		// Process each database connection in parallel using stream
		let db_data_to_snapshot = futures_util::stream::iter(keys_packed.iter().cloned())
			.map(|key_packed| {
				async move {
					let hex_key = hex::encode(&**key_packed);

					// Acquire the connection
					//
					// We don't lock the entry because we can't hold an scc lock in a multithreaded context.
					// However, we hold the lock to the `SqliteConn.conn` which prevents concurrent snapshots
					// that would cause conflict or out-of-order writes.
					let (conn, prev_snapshotted_state) = if let Some(x) = self
						.writer_pools
						.read_async(&key_packed, |_, v| {
							(v.conn_once.get().cloned(), v.snapshotted_state.clone())
						})
						.await
					{
						x
					} else {
						if ensure_exists {
							bail!("attempting to snapshot database that's not loaded");
						} else {
							tracing::debug!(key=?hex_key, "skipping snapshot, database is not loaded");
							return GlobalResult::Ok(None);
						}
					};

					let conn = match conn {
						Some(conn) => conn,
						None => return Ok(None), // Conn will be None if it has not been initiated yet
					};

					// Hold a lock to the connection so nobody else can modify the database while we snapshot
					// it
					let mut conn_raw = conn.conn.lock().await;

					tracing::debug!(key=?hex_key, "snapshotting sqlite database");

					// Get current state
					let current_state = SqliteSnapshottedState {
						total_changes: sqlx::query_scalar("SELECT total_changes()")
							.fetch_one(&mut *conn_raw)
							.await
							.map_err(Error::BuildSqlx)?,
						schema_version: sqlx::query_scalar("PRAGMA schema_version")
							.fetch_one(&mut *conn_raw)
							.await
							.map_err(Error::BuildSqlx)?,
					};

					// Compare with last snapshot state
					ensure!(
						prev_snapshotted_state.total_changes <= current_state.total_changes,
						"total_changes() went down"
					);
					ensure!(
						prev_snapshotted_state.schema_version <= current_state.schema_version,
						"schema_version() went down"
					);
					if prev_snapshotted_state == current_state {
						tracing::debug!(key=?hex_key, "no changes detected, skipping sqlite database snapshot");
						return Ok(None);
					}

					tracing::debug!(
						key=?hex_key,
						?prev_snapshotted_state,
						?current_state,
						"detected changes in sqlite database"
					);

					// Flush WAL journal
					sqlx::query("PRAGMA wal_checkpoint(TRUNCATE);")
						.execute(&mut *conn_raw)
						.await?;

					// Stream the database file and compress it
					let mut compressed_data = Vec::new();
					let file = tokio::fs::File::open(&conn.db_path)
						.await
						.map_err(Error::Io)?;
					let mut reader = tokio::io::BufReader::new(file);
					let mut encoder = lz4_flex::frame::FrameEncoder::new(&mut compressed_data);

					let mut buffer = [0u8; 16 * 1024]; // 16 KiB
					loop {
						let bytes_read = reader.read(&mut buffer).await.map_err(Error::Io)?;
						if bytes_read == 0 {
							break;
						}
						encoder
							.write_all(&buffer[..bytes_read])
							.map_err(Error::Io)?;
					}
					encoder.finish().map_err(Error::Lz4)?;

					// 3 MiB
					if compressed_data.len() > 3 * 1024 * 1024 {
						metrics::SQLITE_LARGE_DB
							.with_label_values(&[&hex_key])
							.set(compressed_data.len().try_into().unwrap_or(i64::MAX));
					}

					Ok(Some((
						key_packed,
						Arc::new(compressed_data),
						current_state,
						prev_snapshotted_state,
					)))
				}
			})
			.buffer_unordered(32)
			.try_filter_map(|result| async move { Ok(result) })
			.try_collect::<Vec<_>>()
			.await?;

		// If no databases need to be snapshotted, return early
		if db_data_to_snapshot.is_empty() {
			return Ok(false);
		}

		// Write to FDB in a single transaction
		let fdb = unwrap!(self.fdb.as_ref());
		fdb.run(|tx, _mc| {
			let db_data_to_snapshot = db_data_to_snapshot.clone();
			let subspace = self.subspace.clone();
			async move {
				for (key_packed, data, _, _) in &db_data_to_snapshot {
					// Clear previous data
					let db_data_subspace =
						subspace.subspace(&keys::DbDataKey::new(key_packed.clone()));
					tx.clear_subspace_range(&db_data_subspace);
					let compressed_db_data_subspace =
						subspace.subspace(&keys::CompressedDbDataKey::new(key_packed.clone()));
					tx.clear_subspace_range(&compressed_db_data_subspace);

					// Write chunks
					for (idx, chunk) in data.chunks(CHUNK_SIZE).enumerate() {
						let chunk_key = keys::CompressedDbDataChunkKey {
							db_name_segment: key_packed.clone(),
							chunk: idx,
						};

						tx.set(&subspace.pack(&chunk_key), chunk);
					}
				}

				Ok(())
			}
			.instrument(tracing::info_span!("snapshot_sqlite_write_tx"))
		})
		.await?;

		let dt = start_instant.elapsed().as_secs_f64();
		let total_data_size = db_data_to_snapshot
			.iter()
			.map(|(_, data, _, _)| data.len())
			.sum::<usize>() as f64;

		// Update state if write was successful
		for (key_packed, data, current_state, prev_snapshotted_state) in db_data_to_snapshot {
			let hex_key = hex::encode(&**key_packed);

			// Because this was batch processed we don't know the rate for each individual key, just estimate
			// by calculating the size ratio
			let ratio = data.len() as f64 / total_data_size;
			metrics::SQLITE_UPLOAD_DB_RATE
				.with_label_values(&[&hex_key])
				.set(data.len() as f64 / (dt * ratio));

			self.writer_pools
				.update_async(&key_packed, |_, v| {
					// Validate state
					if v.snapshotted_state != prev_snapshotted_state {
						tracing::error!(
							current = ?v.snapshotted_state,
							expected = ?prev_snapshotted_state,
							"snapshotted state changed unexpectedly, indicating a potential race condition"
						);
					} else {
						// Update snapshot
						v.snapshotted_state = current_state;
					}
				})
				.await;
		}

		Ok(true)
	}

	/// GC loop for SqlitePoolManager
	#[tracing::instrument(skip_all)]
	async fn manager_gc_loop(self: Arc<Self>, mut shutdown: broadcast::Receiver<()>) {
		let mut interval = tokio::time::interval(GC_INTERVAL);

		loop {
			tokio::select! {
				_ = interval.tick() => {},
				Ok(_) = shutdown.recv() => {
					tracing::debug!("shutting down sqlite pool manager");
					break;
				}
			}

			// Anything last used before this instant will be removed
			let expire_ts = Instant::now() - POOL_TTL;

			// Find expired entries to remove
			//
			// We do this by collecting keys instead of using `retain` or `prune` since we only
			// want to remove the entry if it's successfully snapshotted. If it's not, it should be
			// retained in the map until successfully snapshotted.
			let mut to_remove = Vec::new();
			let mut total_count = 0;
			self.writer_pools.scan_async(|k, v| {
				total_count += 1;
				if v.last_access <= expire_ts {
					if let Some(conn) = v.conn_once.get() {
						// Validate that this is the only reference to the database
						let ref_count = Arc::strong_count(conn);
						if ref_count == 1 {
							to_remove.push(k.clone());
						} else {
							tracing::warn!(?ref_count, ?k, "sqlite pool is expired and should have no references, but still has references");
						}
					} else {
						tracing::warn!(?k, "sqlite pool is expired but conn was never acquired");
						to_remove.push(k.clone());
					}
				}
			}).await;

			// Evict each entry
			let mut removed = 0;
			for key in to_remove {
				match self.evict_with_key(&[key.clone()]).await {
					Ok(_) => {
						removed += 1;
					}
					Err(err) => {
						tracing::error!(?err, ?key, "failed to evict sqlite db, will retry later");
					}
				}
			}

			tracing::debug!(?removed, total = ?total_count, "gc sqlite pools");
		}
	}
}

// MARK: FDB Helpers
impl SqlitePoolManager {
	/// Returns true if db exists, false if not.
	#[tracing::instrument(name = "sqlite_read_from_fdb", skip_all)]
	async fn read_from_fdb(&self, key_packed: KeyPacked, db_path: &Path) -> GlobalResult<bool> {
		let hex_key = hex::encode(&*key_packed);
		let fdb = unwrap!(self.fdb.as_ref());

		let start_instant = Instant::now();

		let (data, chunks) = fdb
			.run(|tx, _mc| {
				let key_packed = key_packed.clone();
				async move {
					let compressed_db_data_subspace = self
						.subspace
						.subspace(&keys::CompressedDbDataKey::new(key_packed.clone()));

					// Fetch all chunks
					let mut compressed_data_stream = tx.get_ranges_keyvalues(
						fdb::RangeOption {
							mode: StreamingMode::WantAll,
							..(&compressed_db_data_subspace).into()
						},
						SERIALIZABLE,
					);

					// Aggregate data
					let mut buf = Vec::new();
					let mut chunk_count = 0;

					let mut compressed_data_buf = Vec::new();
					while let Some(entry) = compressed_data_stream.try_next().await? {
						// Parse key
						let key = self
							.subspace
							.unpack::<keys::CompressedDbDataChunkKey>(entry.key())
							.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

						// Validate chunk
						if chunk_count != key.chunk {
							return Err(FdbBindingError::CustomError(
								SqliteFdbError::MismatchedChunk {
									chunk_count,
									key_idx: key.chunk,
								}
								.into(),
							));
						}
						chunk_count += 1;

						// Write to buffer
						compressed_data_buf.extend(entry.value());
					}

					// Decompress the data
					let mut decoder = lz4_flex::frame::FrameDecoder::new(&compressed_data_buf[..]);
					decoder
						.read_to_end(&mut buf)
						.map_err(Error::Io)
						.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

					// If there is no compressed data, read from the uncompressed data (backwards compatibility)
					if chunk_count == 0 {
						let db_data_subspace = self
							.subspace
							.subspace(&keys::DbDataKey::new(key_packed.clone()));
						let mut data_stream = tx.get_ranges_keyvalues(
							fdb::RangeOption {
								mode: StreamingMode::WantAll,
								..(&db_data_subspace).into()
							},
							SERIALIZABLE,
						);

						while let Some(entry) = data_stream.try_next().await? {
							// Parse key
							let key = self
								.subspace
								.unpack::<keys::DbDataChunkKey>(entry.key())
								.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

							// Validate chunk
							if chunk_count != key.chunk {
								return Err(FdbBindingError::CustomError(
									SqliteFdbError::MismatchedChunk {
										chunk_count,
										key_idx: key.chunk,
									}
									.into(),
								));
							}
							chunk_count += 1;

							// Write to buffer
							buf.extend(entry.value());
						}
					}

					Ok((buf, chunk_count))
				}
				.instrument(tracing::info_span!("read_from_fdb_tx"))
			})
			.await?;

		if chunks > 0 {
			let data_len = data.len();
			tracing::debug!(key=?hex_key, ?chunks, ?data_len, "loaded database from fdb");

			tokio::fs::write(db_path, data).await.map_err(Error::Io)?;

			let dt = start_instant.elapsed();
			metrics::SQLITE_DOWNLOAD_DB_RATE
				.with_label_values(&[&hex_key])
				.set(data_len as f64 / dt.as_secs_f64());

			Ok(true)
		} else {
			tracing::debug!(key=?hex_key, "db not found in fdb");

			Ok(false)
		}
	}
}

impl Drop for SqlitePoolManager {
	fn drop(&mut self) {
		// Ignore send errors - receivers may already be dropped
		let _ = self.shutdown.send(());
	}
}

type SqliteConnHandle = Arc<SqliteConn>;

pub struct SqliteConn {
	key_packed: KeyPacked,
	conn: Mutex<sqlx::SqliteConnection>,
	db_path: PathBuf,
	manager: SqlitePoolManagerHandle,

	/// Used to notify future when this is dropped.
	_drop_task: oneshot::Sender<()>,
}

impl SqliteConn {
	#[tracing::instrument(name = "sqlite_conn_connect", skip_all)]
	async fn connect(
		key_packed: KeyPacked,
		conn_type: SqliteConnType,
		manager: SqlitePoolManagerHandle,
	) -> Result<SqliteConnHandle, Error> {
		// Derive db path & URL
		let hex_key_str = hex::encode(&*key_packed);
		let (db_path, db_url) = match &manager.storage {
			SqliteStorage::Local { path } => {
				// Determine the persistent location of this database
				let db_path = path.join(format!("{hex_key_str}.db"));
				let db_url = format!("sqlite://{}", db_path.display());
				(db_path, db_url)
			}
			SqliteStorage::FoundationDb => {
				let sqlite_path = dirs::data_local_dir()
					.ok_or_else(|| {
						Error::Io(io::Error::new(
							io::ErrorKind::NotFound,
							"Could not determine local data directory",
						))
					})?
					.join("rivet-sqlite");

				// Ensure the directory exists
				tokio::fs::create_dir_all(&sqlite_path)
					.await
					.map_err(Error::Io)?;

				// Generate temporary file location so multiple readers don't clobber each other
				let db_path =
					sqlite_path.join(format!("rivet-sqlite-{hex_key_str}-{}.db", Uuid::new_v4()));
				let db_url = format!("sqlite://{}", db_path.display());
				(db_path, db_url)
			}
		};

		// Load database
		match &manager.storage {
			SqliteStorage::Local { .. } => {
				if !Sqlite::database_exists(&db_url)
					.await
					.map_err(Error::BuildSqlx)?
				{
					tracing::debug!(?db_url, "sqlite database does not exist");
					if conn_type.is_reader() {
						return Err(Error::Global(
							AssertionError::Panic {
								message: "cannot open reader for database that doesn't exist"
									.into(),
								location: global_error::location!(),
							}
							.into(),
						));
					}
				} else {
					tracing::debug!(?db_url, "sqlite database already exists");
				}
			}
			SqliteStorage::FoundationDb => {
				// Read db from FDB
				let db_exists = manager
					.read_from_fdb(key_packed.clone(), &db_path)
					.await
					.map_err(Error::Global)?;

				// Create database if needed
				if !db_exists {
					tracing::debug!(?db_url, "sqlite database does not exist");
					if conn_type.is_reader() {
						return Err(Error::Global(
							AssertionError::Panic {
								message: "cannot open reader for database that doesn't exist"
									.into(),
								location: global_error::location!(),
							}
							.into(),
						));
					}
				} else {
					tracing::debug!(?db_url, "sqlite database already exists");
				}
			}
		};

		tracing::debug!(?db_url, "sqlite connecting");

		// Connect to database
		//
		// We don't need a connection pool since we only have one reader/writer at a time
		let res = if conn_type.is_writer() {
			db_url
				.parse::<SqliteConnectOptions>()
				.map_err(Error::BuildSqlx)?
				.create_if_missing(true)
				// Set busy timeout to 5 seconds to avoid "database is locked" errors
				.busy_timeout(Duration::from_secs(5))
				// Enable foreign key constraint enforcement
				.foreign_keys(true)
				// Enable auto vacuuming and set it to incremental mode for gradual space reclaiming
				.auto_vacuum(SqliteAutoVacuum::Incremental)
				// Set synchronous mode to NORMAL for performance and data safety balance
				.synchronous(SqliteSynchronous::Normal)
				.journal_mode(SqliteJournalMode::Wal)
				.connect()
				.await
		} else {
			db_url
				.parse::<SqliteConnectOptions>()
				.map_err(Error::BuildSqlx)?
				.read_only(true)
				// Set busy timeout to 5 seconds to avoid "database is locked" errors
				.busy_timeout(Duration::from_secs(5))
				// Enable foreign key constraint enforcement
				.foreign_keys(true)
				// Set synchronous mode to NORMAL for performance and data safety balance
				.synchronous(SqliteSynchronous::Normal)
				.connect()
				.await
		};

		let conn_raw = match res {
			Ok(x) => x,
			Err(err) => {
				tracing::error!(
					?conn_type,
					?key_packed,
					?db_url,
					"failed to connect to sqlite"
				);
				return Err(Error::BuildSqlx(err));
			}
		};

		tracing::debug!(?db_url, "sqlite connected");

		let conn_type_str = if conn_type.is_writer() {
			"writer"
		} else {
			"reader"
		};
		metrics::SQLITE_POOL_SIZE
			.with_label_values(&[conn_type_str])
			.inc();

		// Create drop handle
		let (drop_tx, drop_rx) = oneshot::channel();
		tokio::spawn({
			let manager = manager.clone();
			let db_path = db_path.clone();

			async move {
				// Wait for drop signal
				let _ = drop_rx.await;

				// Remove file
				match manager.storage {
					SqliteStorage::Local { .. } => {}
					SqliteStorage::FoundationDb => {
						if let Err(err) = tokio::fs::remove_file(&db_path).await {
							tracing::warn!(
								?err,
								?db_path,
								"failed to remove temporary sqlite db file on drop"
							);
						}

						if let Err(err) =
							tokio::fs::remove_file(format!("{}-shm", db_path.display())).await
						{
							tracing::debug!(
								?err,
								?db_path,
								"failed to remove temporary sqlite db shm file on drop"
							);
						}

						if let Err(err) =
							tokio::fs::remove_file(format!("{}-wal", db_path.display())).await
						{
							tracing::debug!(
								?err,
								?db_path,
								"failed to remove temporary sqlite db wal file on drop"
							);
						}
					}
				}

				metrics::SQLITE_POOL_SIZE
					.with_label_values(&[conn_type_str])
					.dec();
			}
			.instrument(tracing::info_span!("sqlite_conn_drop"))
		});

		Ok(Arc::new(SqliteConn {
			key_packed,
			conn: Mutex::new(conn_raw),
			db_path,
			manager,
			_drop_task: drop_tx,
		}))
	}
}

impl SqliteConn {
	#[tracing::instrument(name = "sqlite_conn_snapshot", skip_all)]
	pub async fn snapshot(&self) -> GlobalResult<bool> {
		match self
			.manager
			.snapshot_with_key(&[self.key_packed.clone()], true)
			.await
		{
			Ok(x) => Ok(x),
			Err(err) => {
				tracing::error!(
					?err,
					"failed to snapshot, will attempt snapshotting again when gc'd"
				);
				Ok(false)
			}
		}
	}
}

// HACK: Implement mock methods to make this act like an SQLite pool so it can be used with the SQL
// macros.
impl SqliteConn {
	pub fn try_acquire(&self) -> Option<tokio::sync::MutexGuard<'_, sqlx::SqliteConnection>> {
		self.conn.try_lock().ok()
	}

	#[tracing::instrument(name = "sqlite_acquire", skip_all)]
	pub async fn acquire(
		&self,
	) -> Result<tokio::sync::MutexGuard<'_, sqlx::SqliteConnection>, sqlx::Error> {
		Ok(self.conn.lock().await)
	}

	#[tracing::instrument(name = "sqlite_conn", skip_all)]
	pub async fn conn(
		&self,
	) -> Result<tokio::sync::MutexGuard<'_, sqlx::SqliteConnection>, sqlx::Error> {
		self.acquire().await
	}
}
