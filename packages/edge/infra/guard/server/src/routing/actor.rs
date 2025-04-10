use chirp_workflow::prelude::*;
use cluster::types::GuardPublicHostname;
use fdb_util::{FormalKey, SNAPSHOT};
use foundationdb::{self as fdb};
use global_error::GlobalResult;
use pegboard::types::EndpointType;
use regex::Regex;
use rivet_guard_core::proxy_service::{RouteTarget, RoutingResult, RoutingTimeout};
use uuid::Uuid;
// Use pegboard module
use pegboard::util::build_actor_hostname_and_path_regex;

/// Route requests to actor services based on hostname and path
pub async fn route_actor_request(
	ctx: &StandaloneCtx,
	host: &str,
	path: &str,
	dc_id: Uuid,
) -> GlobalResult<Option<RoutingResult>> {
	// Get DC information for the current datacenter
	let dc_res = ctx
		.op(cluster::ops::datacenter::get::Input {
			datacenter_ids: vec![dc_id],
		})
		.await?;
	let dc = unwrap!(dc_res.datacenters.first());

	// Get the guard public hostname from the datacenter config
	let guard_public_hostname = &dc.guard_public_hostname;

	// Try routing with Hostname endpoint type
	if let Some(routing_result) = try_route_with_endpoint_type(
		ctx,
		host,
		path,
		EndpointType::Hostname,
		guard_public_hostname,
		dc_id,
	)
	.await?
	{
		return Ok(Some(routing_result));
	}

	// Try routing with Path endpoint type
	if let Some(routing_result) = try_route_with_endpoint_type(
		ctx,
		host,
		path,
		EndpointType::Path,
		guard_public_hostname,
		dc_id,
	)
	.await?
	{
		return Ok(Some(routing_result));
	}

	// No matching route found
	Ok(None)
}

/// Try to route the request using the specified endpoint type
async fn try_route_with_endpoint_type(
	ctx: &StandaloneCtx,
	hostname: &str,
	path: &str,
	endpoint_type: EndpointType,
	guard_public_hostname: &GuardPublicHostname,
	dc_id: Uuid,
) -> GlobalResult<Option<RoutingResult>> {
	// Build regexes for the endpoint type
	let (hostname_regex, path_regex) =
		match build_actor_hostname_and_path_regex(endpoint_type, guard_public_hostname) {
			Ok(Some(regexes)) => regexes,
			Ok(None) => return Ok(None),
			Err(e) => {
				tracing::error!(
					?endpoint_type,
					"Failed to build actor hostname and path regex: {}",
					e
				);
				return Ok(None);
			}
		};

	// Check if hostname matches the pattern
	if !hostname_regex.is_match(hostname) {
		return Ok(None);
	}

	// Extract actor_id and port_name based on the endpoint type
	let (actor_id, port_name) = match endpoint_type {
		EndpointType::Hostname => {
			// For hostname-based routing, extract from hostname
			if let Some(captures) = hostname_regex.captures(hostname) {
				match (captures.name("actor_id"), captures.name("port_name")) {
					(Some(actor_id), Some(port_name)) => match Uuid::parse_str(actor_id.as_str()) {
						Ok(actor_id) => (actor_id, port_name.as_str().to_string()),
						Err(_) => return Ok(None),
					},
					_ => return Ok(None),
				}
			} else {
				return Ok(None);
			}
		}
		EndpointType::Path => {
			// For path-based routing, verify hostname and extract from path
			if !hostname_regex.is_match(hostname) {
				return Ok(None);
			}

			// Get the path_regex (should exist for path-based routing)
			let path_regex = match path_regex {
				Some(re) => re,
				None => {
					tracing::error!("Path regex is missing for path-based routing");
					return Ok(None);
				}
			};

			if let Some(captures) = path_regex.captures(path) {
				match (captures.name("actor_id"), captures.name("port_name")) {
					(Some(actor_id), Some(port_name)) => match Uuid::parse_str(actor_id.as_str()) {
						Ok(actor_id) => (actor_id, port_name.as_str().to_string()),
						Err(_) => return Ok(None),
					},
					_ => return Ok(None),
				}
			} else {
				return Ok(None);
			}
		}
	};

	// Build the path for the route target based on endpoint type
	let path_to_forward = match endpoint_type {
		EndpointType::Hostname => path.to_string(),
		EndpointType::Path => {
			// For path-based routing, we need to remove the actor prefix from the path
			let prefix = format!("/{}-{}", actor_id, port_name);
			if path.starts_with(&prefix) {
				if path.len() > prefix.len() {
					path[prefix.len()..].to_string()
				} else {
					"/".to_string()
				}
			} else {
				path.to_string()
			}
		}
	};

	// Now that we have the actor_id and port_name, lookup the actor
	match find_actor(ctx, &actor_id, &port_name, dc_id, path_to_forward).await? {
		Some(mut target) => Ok(Some(RoutingResult {
			targets: vec![target],
			timeout: RoutingTimeout {
				routing_timeout: 10,
			},
		})),
		None => Ok(None),
	}
}

/// Find an actor by actor_id and port_name - this would call into the actor registry
async fn find_actor(
	ctx: &StandaloneCtx,
	actor_id: &Uuid,
	port_name: &str,
	dc_id: Uuid,
	path_to_forward: String,
) -> GlobalResult<Option<RouteTarget>> {
	// Fetch ports
	let proxied_ports = ctx
		.fdb()
		.await?
		.run(|tx, _mc| async move {
			// NOTE: This is not SERIALIZABLE because we don't want to conflict with port updates
			// and its not important if its slightly stale
			let proxied_ports_key = pegboard::keys::actor::ProxiedPortsKey::new(*actor_id);
			let raw = tx
				.get(
					&pegboard::keys::subspace().pack(&proxied_ports_key),
					SNAPSHOT,
				)
				.await?;
			if let Some(raw) = raw {
				Ok(Some(proxied_ports_key.deserialize(&raw).map_err(|x| {
					fdb::FdbBindingError::CustomError(x.into())
				})?))
			} else {
				Ok(None)
			}
		})
		.await?;

	let Some(proxied_ports) = proxied_ports else {
		// TODO: Check if actor exists
		// TODO: Return error actor not found or not running
		return Ok(None);
	};

	// Find the port
	let Some(proxied_port) = proxied_ports.iter().find(|pp| pp.port_name == port_name) else {
		// TODO: Return error port not found
		return Ok(None);
	};

	// TODO: Validate protocol based on the incoming port

	Ok(Some(RouteTarget {
		actor_id: Some(*actor_id),
		server_id: None,
		host: proxied_port.lan_hostname.parse()?,
		port: proxied_port.source,
		path: path_to_forward,
	}))
}
