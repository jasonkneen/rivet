// NOTE: Requires installing skopeo and umoci on the machine running this test

use std::{
	sync::{
		atomic::{AtomicBool, AtomicI32, Ordering},
		Arc,
	},
	time::Duration,
};

use futures_util::StreamExt;
use nix::{
	sys::signal::{kill, Signal},
	unistd::Pid,
};
use pegboard::protocol;
use pegboard_manager::Ctx;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::tungstenite::protocol::Message;
use uuid::Uuid;

mod common;
use common::*;

/// Verifies client state is correct after manager is stopped, actor is killed, manager is restarted.
#[tokio::test(flavor = "multi_thread")]
async fn client_state_external_kill() {
	setup_tracing();

	tracing::info!("starting test");

	let (_gen_tmp_dir, gen_tmp_dir_path) = setup_dependencies().await;

	let ctx_wrapper: Arc<Mutex<Option<Arc<Ctx>>>> = Arc::new(Mutex::new(None));
	let (close_tx, mut close_rx) = tokio::sync::watch::channel(());
	let close_tx = Arc::new(close_tx);

	let actor_id = Uuid::new_v4();
	let first_client = Arc::new(AtomicBool::new(true));
	let actor_pid = Arc::new(AtomicI32::new(0));

	let port = portpicker::pick_unused_port().expect("no free ports");
	let first_client2 = first_client.clone();
	let actor_pid2 = actor_pid.clone();
	start_server(
		ctx_wrapper.clone(),
		close_tx,
		port,
		move |ctx_wrapper, close_tx, raw_stream| {
			handle_connection(
				ctx_wrapper,
				close_tx,
				raw_stream,
				actor_id,
				first_client2.clone(),
				actor_pid2.clone(),
			)
		},
	);

	// Init project directories
	let tmp_dir = tempfile::TempDir::new().unwrap();
	let config = init_client(&gen_tmp_dir_path, tmp_dir.path()).await;
	tracing::info!(path=%tmp_dir.path().display(), "client dir");

	start_client(config.clone(), ctx_wrapper.clone(), close_rx.clone(), port).await;

	first_client.store(false, Ordering::SeqCst);
	close_rx.mark_unchanged();

	let pid = actor_pid.load(Ordering::SeqCst);
	if pid == 0 {
		panic!("pid never set");
	}

	// Kill actor
	tracing::info!("killing actor");
	kill(Pid::from_raw(pid), Signal::SIGKILL).unwrap();

	start_client(config, ctx_wrapper, close_rx, port).await;
}

async fn handle_connection(
	ctx_wrapper: Arc<Mutex<Option<Arc<Ctx>>>>,
	close_tx: Arc<tokio::sync::watch::Sender<()>>,
	raw_stream: TcpStream,
	actor_id: Uuid,
	first_client: Arc<AtomicBool>,
	actor_pid: Arc<AtomicI32>,
) {
	tokio::spawn(async move {
		let ws_stream = tokio_tungstenite::accept_async(raw_stream).await.unwrap();
		let (mut tx, mut rx) = ws_stream.split();

		tokio::time::sleep(std::time::Duration::from_millis(16)).await;

		// Read ctx from wrapper
		let ctx = {
			let guard = ctx_wrapper.lock().await;
			guard.clone().unwrap()
		};

		// Receive messages from socket
		while let Some(msg) = rx.next().await {
			match msg.unwrap() {
				Message::Binary(buf) => {
					let protocol_version = 1;
					let packet = protocol::ToServer::deserialize(protocol_version, &buf).unwrap();

					match packet {
						protocol::ToServer::Init { .. } => {
							send_init_packet(&mut tx).await;

							if first_client.load(Ordering::SeqCst) {
								// Spawn actor on first client
								start_echo_actor(&mut tx, actor_id).await;
							}
						}
						protocol::ToServer::Events(events) => {
							for event in events {
								tracing::info!(?event, "received event");

								let protocol::Event::ActorStateUpdate { state, .. } =
									event.inner.deserialize().unwrap();

								match state {
									// Wait for actor to start running
									protocol::ActorState::Running { pid, .. } => {
										// Save pid
										actor_pid.store(pid as i32, Ordering::SeqCst);

										// Stop first client
										close_tx.send(()).unwrap();
									}
									protocol::ActorState::Exited { .. } => {
										tokio::time::sleep(Duration::from_millis(5)).await;

										// Verify client state
										let actors = ctx.actors().read().await;
										assert!(
											!actors.contains_key(&(actor_id, 0)),
											"actor still in client memory"
										);

										// Test complete
										close_tx.send(()).unwrap();
									}
									protocol::ActorState::Starting
									| protocol::ActorState::Stopped => {}
									state => panic!("unexpected state received: {state:?}"),
								}
							}
						}
						protocol::ToServer::AckCommands { .. } => {}
					}
				}
				Message::Close(_) => {
					panic!("socket closed");
				}
				_ => {}
			}
		}

		tracing::info!("client disconnected");
	});
}
