use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub service_name: String,
    pub status: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceResponse {
    pub service: String,
    pub status: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

pub trait ConxianService {
    fn name(&self) -> &str;
    fn status(&self) -> ServiceStatus;
    fn handle_request(&self, payload: &str) -> ServiceResponse;
}
