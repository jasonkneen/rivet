use futures_util::{StreamExt, TryStreamExt};
use indoc::indoc;
use proto::backend::pkg::*;
use rivet_operation::prelude::*;

pub async fn start(config: rivet_config::Config, pools: rivet_pools::Pools) -> GlobalResult<()> {
	run_from_env(config, pools, util::timestamp::now()).await
}

#[tracing::instrument(skip_all)]
pub async fn run_from_env(
	config: rivet_config::Config,
	pools: rivet_pools::Pools,
	ts: i64,
) -> GlobalResult<()> {
	let client =
		chirp_client::SharedClient::from_env(pools.clone())?.wrap_new("user-delete-pending");
	let cache = rivet_cache::CacheInner::from_env(&config, pools.clone())?;
	let ctx = OperationContext::new(
		"user-delete-pending".into(),
		std::time::Duration::from_secs(60),
		config,
		rivet_connection::Connection::new(client, pools, cache),
		Uuid::new_v4(),
		Uuid::new_v4(),
		util::timestamp::now(),
		util::timestamp::now(),
		(),
	);

	let user_ids = sql_fetch_all!(
		[ctx, (Uuid,)]
		"
		SELECT user_id
		FROM db_user.users
		WHERE delete_request_ts < $1
		",
		ts - util::duration::days(30),
	)
	.await?
	.into_iter()
	.map(|(user_id,)| user_id)
	.collect::<Vec<_>>();

	tracing::info!(count = user_ids.len(), "publishing deletes");

	futures_util::stream::iter(user_ids.into_iter())
		.map(|user_id| {
			msg!([ctx] user::msg::delete(user_id) -> user::msg::delete_complete {
				user_id: Some(user_id.into()),
			})
		})
		.buffer_unordered(32)
		.try_collect::<Vec<_>>()
		.await?;

	Ok(())
}
