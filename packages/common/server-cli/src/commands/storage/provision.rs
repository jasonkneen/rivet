use anyhow::*;
use clap::Parser;

use rivet_service_manager::RunConfig;

#[derive(Parser)]
pub struct Opts {}

impl Opts {
	pub async fn execute(self, config: rivet_config::Config, run_config: &RunConfig) -> Result<()> {
		s3_util::provision(config, &run_config.s3_buckets).await?;
		Ok(())
	}
}
