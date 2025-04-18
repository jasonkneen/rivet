use chirp_workflow::prelude::*;
use ds::types;
use rivet_operation::prelude::proto::backend;
use serde_json::json;

#[workflow_test]
async fn server_create(ctx: TestCtx) {
	let game_res = op!([ctx] faker_game {
		..Default::default()
	})
	.await
	.unwrap();
	let env_id = game_res.prod_env_id.unwrap();

	ctx.op(ds::ops::game_config::upsert::Input {
		game_id: game_res.game_id.unwrap().as_uuid(),
		host_networking_enabled: Some(true),
		..Default::default()
	})
	.await
	.unwrap();

	// Pick an existing cluster
	let cluster_id = ctx
		.op(cluster::ops::list::Input {})
		.await
		.unwrap()
		.cluster_ids
		.first()
		.unwrap()
		.to_owned();

	let build_res = op!([ctx] faker_build {
		env_id: Some(env_id),
		image: backend::faker::Image::JsEcho as i32,
	})
	.await
	.unwrap();

	let faker_region = op!([ctx] faker_region {}).await.unwrap();

	let env = vec![
		("some_envkey_test".to_string(), "2134523".to_string()),
		(
			"some_other_envkey_test".to_string(),
			"4325234356".to_string(),
		),
	]
	.into_iter()
	.collect();

	let ports = vec![(
		"ds_testing2".to_string(),
		ds::workflows::server::Port {
			internal_port: None,
			routing: types::Routing::GameGuard {
				protocol: types::GameGuardProtocol::Http,
				authorization: types::PortAuthorization::None,
			},
		},
	)]
	// Collect into hashmap
	.into_iter()
	.collect();

	let server_id = Uuid::new_v4();

	let mut sub = ctx
		.subscribe::<ds::workflows::server::CreateComplete>(&json!({
			"server_id": server_id,
		}))
		.await
		.unwrap();

	ctx.workflow(ds::workflows::server::Input {
		server_id,
		env_id: *env_id,
		datacenter_id: faker_region.region_id.unwrap().as_uuid(),
		cluster_id,
		runtime: ds::types::ServerRuntime::Pegboard,
		resources: ds::types::ServerResources {
			cpu_millicores: 100,
			memory_mib: 200,
		},
		lifecycle: ds::types::ServerLifecycle {
			kill_timeout_ms: 0,
			durable: false,
		},
		tags: vec![(String::from("test"), String::from("123"))]
			.into_iter()
			.collect(),
		root_user_enabled: false,
		args: Vec::new(),
		environment: env,
		image_id: build_res.build_id.unwrap().as_uuid(),
		network_mode: types::NetworkMode::Host,
		network_ports: ports,
	})
	.tag("server_id", server_id)
	.dispatch()
	.await
	.unwrap();

	sub.next().await.unwrap();

	tracing::info!("waiting for public hostname");

	let (hostname, port) = loop {
		tokio::time::sleep(std::time::Duration::from_secs(1)).await;

		let server = ctx
			.op(ds::ops::server::get::Input {
				server_ids: vec![server_id],
				endpoint_type: Some(ds::types::EndpointType::Hostname),
			})
			.await
			.unwrap()
			.servers
			.into_iter()
			.next()
			.unwrap();

		let port = server.network_ports.get("ds_testing2").unwrap();

		if let Some(hostname) = port.public_hostname.as_ref() {
			break (hostname.clone(), port.public_port.unwrap());
		}
	};

	tracing::info!(%hostname, %port, "echoing");

	// Echo body
	let random_body = Uuid::new_v4().to_string();

	loop {
		// Create a new client each time to prevent cache
		let client = reqwest::Client::new();
		let res = tokio::time::timeout(std::time::Duration::from_secs(3), async {
			client
				.post(format!("http://{hostname}:{port}"))
				.body(random_body.clone())
				.send()
				.await
				.unwrap()
		})
		.await;

		let Ok(res) = res else {
			tracing::warn!("timed out for some reason");
			continue;
		};

		if let reqwest::StatusCode::NOT_FOUND = res.status() {
			tracing::warn!("endpoint not found yet");
			tokio::time::sleep(std::time::Duration::from_secs(2)).await;
			continue;
		}

		let res = res.error_for_status().unwrap();
		let res_text = res.text().await.unwrap();
		assert_eq!(random_body, res_text, "echoed wrong response");

		break;
	}

	// assert_eq!(game_res.prod_env_id.unwrap(), server.env_id.unwrap().as_uuid());
}
