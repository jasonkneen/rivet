use chirp_workflow::prelude::*;

pub async fn start(config: rivet_config::Config, pools: rivet_pools::Pools) -> GlobalResult<()> {
	run_from_env(config, pools).await?;

	Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn run_from_env(
	config: rivet_config::Config,
	pools: rivet_pools::Pools,
) -> GlobalResult<()> {
	let reg = cluster::registry()?
		.merge(linode::registry()?)?
		.merge(job_run::registry()?)?;

	let db = db::DatabaseCrdbNats::from_pools(pools.clone())?;
	let worker = Worker::new(reg.handle(), db);

	// Start worker
	worker.start(config, pools).await
}
