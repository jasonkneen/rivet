# CloudBuildSummary

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**build_id** | [**uuid::Uuid**](uuid::Uuid.md) |  | 
**upload_id** | [**uuid::Uuid**](uuid::Uuid.md) |  | 
**display_name** | **String** | Represent a resource's readable display name. | 
**create_ts** | **String** | RFC3339 timestamp | 
**content_length** | **i64** | Unsigned 64 bit integer. | 
**complete** | **bool** | Whether or not this build has completely been uploaded. | 
**tags** | **::std::collections::HashMap<String, String>** | Tags of this build | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


