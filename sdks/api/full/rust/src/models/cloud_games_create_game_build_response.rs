/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 *
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct CloudGamesCreateGameBuildResponse {
	#[serde(rename = "build_id")]
	pub build_id: uuid::Uuid,
	#[serde(rename = "upload_id")]
	pub upload_id: uuid::Uuid,
	#[serde(
		rename = "image_presigned_request",
		skip_serializing_if = "Option::is_none"
	)]
	pub image_presigned_request: Option<Box<crate::models::UploadPresignedRequest>>,
	#[serde(
		rename = "image_presigned_requests",
		skip_serializing_if = "Option::is_none"
	)]
	pub image_presigned_requests: Option<Vec<crate::models::UploadPresignedRequest>>,
}

impl CloudGamesCreateGameBuildResponse {
	pub fn new(build_id: uuid::Uuid, upload_id: uuid::Uuid) -> CloudGamesCreateGameBuildResponse {
		CloudGamesCreateGameBuildResponse {
			build_id,
			upload_id,
			image_presigned_request: None,
			image_presigned_requests: None,
		}
	}
}
