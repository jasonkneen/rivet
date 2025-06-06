/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 *
 * Generated by: https://openapi-generator.tech
 */

///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum ActorsQueryLogStream {
	#[serde(rename = "std_out")]
	StdOut,
	#[serde(rename = "std_err")]
	StdErr,
	#[serde(rename = "all")]
	All,
}

impl ToString for ActorsQueryLogStream {
	fn to_string(&self) -> String {
		match self {
			Self::StdOut => String::from("std_out"),
			Self::StdErr => String::from("std_err"),
			Self::All => String::from("all"),
		}
	}
}

impl Default for ActorsQueryLogStream {
	fn default() -> ActorsQueryLogStream {
		Self::StdOut
	}
}
