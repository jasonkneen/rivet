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
pub struct ProvisionDatacentersGetServersResponse {
	#[serde(rename = "servers")]
	pub servers: Vec<crate::models::ProvisionServer>,
}

impl ProvisionDatacentersGetServersResponse {
	pub fn new(
		servers: Vec<crate::models::ProvisionServer>,
	) -> ProvisionDatacentersGetServersResponse {
		ProvisionDatacentersGetServersResponse { servers }
	}
}
