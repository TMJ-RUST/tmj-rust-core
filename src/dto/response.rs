use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
    #[schema(example = "success")]
    pub status: Option<String>,
    #[schema(example = "Operation completed successfully")]
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    #[schema(example = "00")]
    pub status_code: String,
    #[schema(example = "8c7fae99-7bcb-4c1f-9677-5097a23a0435")]
    pub reff_code: String,
    #[schema(example = "Success")]
    pub message: String,
    #[schema(example = "null")]
    pub message_error: Option<String>,
    pub data: Option<T>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    #[schema(example = 1)]
    pub page: i64,
    #[schema(example = 10)]
    pub limit: i64,
    #[schema(example = 42)]
    pub total_items: i64,
    #[schema(example = 5)]
    pub total_pages: i64,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedData<T> {
    pub items: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, reff_code: String, message: impl Into<String>) -> Self {
        Self {
            status_code: "00".to_string(),
            reff_code,
            message: message.into(),
            message_error: None,
            data: Some(data),
        }
    }
}

impl ApiResponse<()> {
    pub fn success_empty(reff_code: String, message: impl Into<String>) -> Self {
        Self {
            status_code: "00".to_string(),
            reff_code,
            message: message.into(),
            message_error: None,
            data: None,
        }
    }

    pub fn error(
        status_code: impl Into<String>,
        reff_code: String,
        message: impl Into<String>,
        message_error: impl Into<String>,
    ) -> Self {
        Self {
            status_code: status_code.into(),
            reff_code,
            message: message.into(),
            message_error: Some(message_error.into()),
            data: None,
        }
    }
}
