# \ServiceApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_service**](ServiceApi.md#create_service) | **POST** /services | 
[**get_service_by_id**](ServiceApi.md#get_service_by_id) | **GET** /services/{id} | 
[**list_services**](ServiceApi.md#list_services) | **GET** /services | 



## create_service

> models::ServiceDtoOutput create_service(create_service_dto)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_service_dto** | [**CreateServiceDto**](CreateServiceDto.md) |  | [required] |

### Return type

[**models::ServiceDtoOutput**](ServiceDTO_Output.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_service_by_id

> get_service_by_id(id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **f64** |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_services

> Vec<models::ServiceListDtoOutputInner> list_services()


### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::ServiceListDtoOutputInner>**](ServiceListDTO_Output_inner.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

