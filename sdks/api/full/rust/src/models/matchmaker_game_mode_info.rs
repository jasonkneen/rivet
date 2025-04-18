/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 *
 * Generated by: https://openapi-generator.tech
 */

/// MatchmakerGameModeInfo : A game mode that the player can join.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct MatchmakerGameModeInfo {
	/// A human readable short identifier used to references resources. Different than a `uuid` because this is intended to be human readable. Different than `DisplayName` because this should not include special characters and be short.
	#[serde(rename = "game_mode_id")]
	pub game_mode_id: String,
}

impl MatchmakerGameModeInfo {
	/// A game mode that the player can join.
	pub fn new(game_mode_id: String) -> MatchmakerGameModeInfo {
		MatchmakerGameModeInfo { game_mode_id }
	}
}
