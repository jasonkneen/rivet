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
pub struct CloudVersionCdnMiddleware {
	#[serde(rename = "kind")]
	pub kind: Box<crate::models::CloudVersionCdnMiddlewareKind>,
}

impl CloudVersionCdnMiddleware {
	pub fn new(kind: crate::models::CloudVersionCdnMiddlewareKind) -> CloudVersionCdnMiddleware {
		CloudVersionCdnMiddleware {
			kind: Box::new(kind),
		}
	}
}
