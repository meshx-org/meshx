# \WorkloadApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_workload**](WorkloadApi.md#create_workload) | **POST** /v1/workloads | 
[**delete_workload**](WorkloadApi.md#delete_workload) | **DELETE** /v1/workloads/{name} | 
[**list_workloads**](WorkloadApi.md#list_workloads) | **GET** /v1/workloads | 



## create_workload

> models::WorkloadCreateResponseDtoOutput create_workload(workload_create_request_dto)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workload_create_request_dto** | [**WorkloadCreateRequestDto**](WorkloadCreateRequestDto.md) |  | [required] |

### Return type

[**models::WorkloadCreateResponseDtoOutput**](WorkloadCreateResponseDto_Output.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_workload

> delete_workload(name, application_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**application_id** | **f64** | Filter workloads from the given ID. | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_workloads

> list_workloads(application_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**application_id** | **f64** | Filter workloads from the given ID. | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

