use rivet_operation::prelude::*;

macro_rules! spawn_workers {
	([$shared_client:ident, $config:ident, $pools:ident, $cache:ident, $join_set:ident] $($pkg:ident),* $(,)?) => {
		$(
			$pkg::workers::spawn_workers(
				$shared_client.clone(),
				$config.clone(),
				$pools.clone(),
				$cache.clone(),
				&mut $join_set,
			)?;
		)*
	};
}

#[tracing::instrument(skip_all)]
pub async fn start(config: rivet_config::Config, pools: rivet_pools::Pools) -> GlobalResult<()> {
	let shared_client = chirp_client::SharedClient::from_env(pools.clone())?;
	let cache = rivet_cache::CacheInner::from_env(&config, pools.clone())?;

	// Start workers
	let mut join_set = tokio::task::JoinSet::new();
	spawn_workers![
		[shared_client, config, pools, cache, join_set]
		cdn_worker,
		cf_custom_hostname_worker,
		cloud_worker,
		external_worker,
		job_log_worker,
		job_run,
		mm_worker,
		team_invite_worker,
		team_worker,
		upload_worker,
		user_worker,
	];

	// Wait for task to exit
	if let Some(res) = join_set.join_next().await {
		match res? {
			Ok(_) => {
				bail!("worker exited unexpectedly")
			}
			Err(err) => {
				return Err(err);
			}
		}
	}

	bail!("no workers running")
}
