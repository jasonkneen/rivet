use chirp_workflow::prelude::*;

pub mod guard;
pub mod worker;

pub const TUNNEL_API_EDGE_PORT: u16 = 5010;

pub fn create_hook(tunnel_name: &str, initialize_immediately: bool) -> GlobalResult<String> {
	let mut script =
		include_str!("../../files/rivet_create_hook.sh").replace("__TUNNEL_NAME__", tunnel_name);

	if initialize_immediately {
		script.push_str("systemctl start rivet_hook\n");
	}

	Ok(script)
}

pub fn fetch_info(server_token: &str) -> GlobalResult<String> {
	Ok(include_str!("../../files/rivet_fetch_info.sh")
		.replace("__SERVER_TOKEN__", server_token)
		.replace(
			"__TUNNEL_API_EDGE_API__",
			&format!("http://127.0.0.1:{TUNNEL_API_EDGE_PORT}"),
		))
}

pub fn fetch_tunnel_tls(
	initialize_immediately: bool,
	server_token: &str,
	traefik_instance_name: &str,
) -> GlobalResult<String> {
	let mut script = include_str!("../../files/rivet_fetch_tunnel_tls.sh")
		.replace("__TRAEFIK_INSTANCE_NAME__", traefik_instance_name)
		.replace("__SERVER_TOKEN__", server_token)
		.replace(
			"__TUNNEL_API_EDGE_API__",
			&format!("http://127.0.0.1:{TUNNEL_API_EDGE_PORT}"),
		);

	if initialize_immediately {
		// Start timer & run script immediately
		script.push_str(indoc!(
			"
			systemctl start rivet_fetch_tunnel_tls.timer
			systemctl start --no-block rivet_fetch_tunnel_tls.service
			"
		));
	}

	Ok(script)
}

pub fn fetch_gg_tls(server_token: &str, traefik_instance_name: &str) -> GlobalResult<String> {
	let script = include_str!("../../files/rivet_fetch_gg_tls.sh")
		.replace("__TRAEFIK_INSTANCE_NAME__", traefik_instance_name)
		.replace("__SERVER_TOKEN__", server_token)
		.replace(
			"__TUNNEL_API_EDGE_API__",
			&format!("http://127.0.0.1:{TUNNEL_API_EDGE_PORT}"),
		);

	Ok(script)
}

pub fn fetch_api_route(server_token: &str, traefik_instance_name: &str) -> GlobalResult<String> {
	let script = include_str!("../../files/rivet_fetch_api_route.sh")
		.replace("__TRAEFIK_INSTANCE_NAME__", traefik_instance_name)
		.replace("__SERVER_TOKEN__", server_token)
		.replace(
			"__TUNNEL_API_EDGE_API__",
			&format!("http://127.0.0.1:{TUNNEL_API_EDGE_PORT}"),
		);

	Ok(script)
}
