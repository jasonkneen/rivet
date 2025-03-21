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
pub struct ActorCreateActorRuntimeRequest {
    #[serde(rename = "environment", skip_serializing_if = "Option::is_none")]
    pub environment: Option<::std::collections::HashMap<String, String>>,
}

impl ActorCreateActorRuntimeRequest {
    pub fn new() -> ActorCreateActorRuntimeRequest {
        ActorCreateActorRuntimeRequest {
            environment: None,
        }
    }
}


