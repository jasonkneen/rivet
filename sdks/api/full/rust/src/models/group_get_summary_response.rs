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
pub struct GroupGetSummaryResponse {
	#[serde(rename = "group")]
	pub group: Box<crate::models::GroupGroupSummary>,
}

impl GroupGetSummaryResponse {
	pub fn new(group: crate::models::GroupGroupSummary) -> GroupGetSummaryResponse {
		GroupGetSummaryResponse {
			group: Box::new(group),
		}
	}
}
