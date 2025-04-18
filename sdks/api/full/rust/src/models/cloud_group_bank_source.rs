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
pub struct CloudGroupBankSource {
	/// The bank account number of this group's bank source.
	#[serde(rename = "account_number")]
	pub account_number: String,
	/// The bank routing number of this group's bank source.
	#[serde(rename = "routing_number")]
	pub routing_number: String,
}

impl CloudGroupBankSource {
	pub fn new(account_number: String, routing_number: String) -> CloudGroupBankSource {
		CloudGroupBankSource {
			account_number,
			routing_number,
		}
	}
}
