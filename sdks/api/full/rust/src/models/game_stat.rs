/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 *
 * Generated by: https://openapi-generator.tech
 */

/// GameStat : A game statistic.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct GameStat {
	#[serde(rename = "config")]
	pub config: Box<crate::models::GameStatConfig>,
	/// A single overall value of the given statistic.
	#[serde(rename = "overall_value")]
	pub overall_value: f64,
}

impl GameStat {
	/// A game statistic.
	pub fn new(config: crate::models::GameStatConfig, overall_value: f64) -> GameStat {
		GameStat {
			config: Box::new(config),
			overall_value,
		}
	}
}
