use chirp_workflow::prelude::*;
use std::{collections::HashMap, convert::TryInto};

use crate::types;

#[derive(Debug)]
pub struct Input {
	pub build_ids: Vec<Uuid>,
}

#[derive(Debug)]
pub struct Output {
	pub builds: Vec<types::Build>,
}

#[derive(sqlx::FromRow)]
pub(crate) struct BuildRow {
	pub(crate) build_id: Uuid,
	game_id: Option<Uuid>,
	env_id: Option<Uuid>,
	upload_id: Uuid,
	display_name: String,
	image_tag: String,
	create_ts: i64,
	kind: i64,
	compression: i64,
	tags: sqlx::types::Json<Box<serde_json::value::RawValue>>,
}

impl TryInto<types::Build> for BuildRow {
	type Error = GlobalError;

	fn try_into(self) -> GlobalResult<types::Build> {
		Ok(types::Build {
			build_id: self.build_id,
			game_id: self.game_id,
			env_id: self.env_id,
			upload_id: self.upload_id,
			display_name: self.display_name,
			image_tag: self.image_tag,
			create_ts: self.create_ts,
			kind: unwrap!(types::BuildKind::from_repr(self.kind.try_into()?)),
			compression: unwrap!(types::BuildCompression::from_repr(
				self.compression.try_into()?
			)),
			// Filter out null values on tags
			tags: serde_json::from_str::<HashMap<String, Option<String>>>(self.tags.0.get())?
				.into_iter()
				.filter_map(|(k, v)| v.map(|v| (k, v)))
				.collect(),
		})
	}
}

#[operation]
pub async fn build_get(ctx: &OperationCtx, input: &Input) -> GlobalResult<Output> {
	let builds = ctx
		.cache()
		.fetch_all_json("build", input.build_ids.clone(), {
			let ctx = ctx.clone();
			move |mut cache, build_ids| {
				let ctx = ctx.clone();
				async move {
					let rows = sql_fetch_all!(
						[ctx, BuildRow]
						"
						SELECT
							build_id,
							game_id,
							env_id,
							upload_id,
							display_name,
							image_tag,
							create_ts,
							kind,
							compression,
							tags
						FROM db_build.builds
						WHERE build_id = ANY($1)
						",
						&build_ids,
					)
					.await?;

					for row in rows {
						cache.resolve(&row.build_id.clone(), row.try_into()?);
					}

					Ok(cache)
				}
			}
		})
		.await?;

	Ok(Output { builds })
}
