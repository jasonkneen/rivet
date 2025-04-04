use std::collections::HashMap;

use chirp_worker::prelude::*;
use futures_util::stream::{StreamExt, TryStreamExt};
use proto::backend::pkg::*;

#[derive(sqlx::FromRow)]
struct UploadRow {
	upload_id: Uuid,
	bucket: String,
	provider: i64,
}

#[derive(sqlx::FromRow)]
struct FileRow {
	upload_id: Uuid,
	path: String,
}

struct BucketDeletions {
	client: s3_util::Client,
	keys: Vec<String>,
}

#[worker(name = "upload-delete")]
async fn worker(ctx: &OperationContext<upload::msg::delete::Message>) -> GlobalResult<()> {
	let request_id = unwrap_ref!(ctx.request_id).as_uuid();
	let upload_ids = ctx
		.upload_ids
		.iter()
		.map(common::Uuid::as_uuid)
		.collect::<Vec<_>>();

	let uploads = sql_fetch_all!(
		[ctx, UploadRow]
		"
		SELECT upload_id, bucket, provider
		FROM db_upload.uploads
		WHERE upload_id = ANY($1)
		",
		&upload_ids,
	)
	.await?;

	let upload_files = sql_fetch_all!(
		[ctx, FileRow]
		"
		SELECT upload_id, path
		FROM db_upload.upload_files
		WHERE upload_id = ANY($1)
		",
		&upload_ids,
	)
	.await?;

	ctx.cache().purge("upload", upload_ids.clone()).await?;
	ctx.cache().purge("upload_file", upload_ids.clone()).await?;

	// Compile uploads into hashmap by bucket
	let mut deletions: HashMap<String, BucketDeletions> =
		HashMap::with_capacity(upload_files.len());
	for upload_file in upload_files {
		let upload = unwrap!(uploads
			.iter()
			.find(|upload| upload.upload_id == upload_file.upload_id));
		let key = format!("{}/{}", upload_file.upload_id, upload_file.path);

		if let Some(x) = deletions.get_mut(&upload.bucket) {
			x.keys.push(key);
		} else {
			let client = s3_util::Client::with_bucket(ctx.config(), &upload.bucket).await?;

			deletions.insert(
				upload.bucket.clone(),
				BucketDeletions {
					client,
					keys: vec![key],
				},
			);
		}
	}

	let counts = deletions
		.iter()
		.map(|(bucket, deletion)| (bucket, deletion.keys.len()))
		.collect::<HashMap<_, _>>();
	tracing::info!(deletions=?counts, "deleting");

	let deletions = deletions
		.into_values()
		.map(|deletion| {
			let delete = s3_util::aws_sdk_s3::types::Delete::builder()
				.set_objects(Some(
					deletion
						.keys
						.iter()
						.map(|key| {
							s3_util::aws_sdk_s3::types::ObjectIdentifier::builder()
								.key(key)
								.build()
						})
						.collect::<Result<Vec<_>, _>>()?,
				))
				.build()?;

			Ok(deletion
				.client
				.delete_objects()
				.bucket(deletion.client.bucket())
				.delete(delete)
				.send())
		})
		.collect::<GlobalResult<Vec<_>>>()?;

	// Execute batch deletions per bucket
	futures_util::stream::iter(deletions)
		.buffer_unordered(32)
		.try_collect::<Vec<_>>()
		.await?;

	// Mark upload as deleted
	sql_execute!(
		[ctx]
		"
		UPDATE db_upload.uploads
		SET deleted_ts = $2
		WHERE upload_id = ANY($1)
		",
		&upload_ids,
		ctx.ts(),
	)
	.await?;

	msg!([ctx] upload::msg::delete_complete(request_id) {
		request_id: ctx.request_id,
		upload_ids: ctx.upload_ids.clone(),
	})
	.await?;

	Ok(())
}
