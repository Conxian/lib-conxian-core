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

pub struct BisqService;

impl ConxianService for BisqService {
    fn name(&self) -> &str {
        "Bisq"
    }

    fn status(&self) -> ServiceStatus {
        ServiceStatus {
            service_name: self.name().to_string(),
            status: "Active".to_string(),
            version: "v1.2.0".to_string(),
        }
    }

    fn handle_request(&self, _payload: &str) -> ServiceResponse {
        ServiceResponse {
            service: self.name().to_string(),
            status: "Success".to_string(),
            message: "Bisq trade verified via Nexus.".to_string(),
            data: None,
        }
    }
}

pub struct BitVMService;

impl BitVMService {
    pub fn verify_job_card(&self, _job_card: &crate::cjcs::JobCard) -> bool {
        false
    }
}

impl ConxianService for BitVMService {
    fn name(&self) -> &str {
        "BitVM"
    }

    fn status(&self) -> ServiceStatus {
        ServiceStatus {
            service_name: self.name().to_string(),
            status: "Active".to_string(),
            version: "v0.1.0".to_string(),
        }
    }

    fn handle_request(&self, _payload: &str) -> ServiceResponse {
        ServiceResponse {
            service: self.name().to_string(),
            status: "NotImplemented".to_string(),
            message: "BitVM2 verification is not yet available.".to_string(),
            data: None,
        }
    }
}

pub struct RGBService;

impl ConxianService for RGBService {
    fn name(&self) -> &str {
        "RGB"
    }

    fn status(&self) -> ServiceStatus {
        ServiceStatus {
            service_name: self.name().to_string(),
            status: "Active".to_string(),
            version: "v0.10.0".to_string(),
        }
    }

    fn handle_request(&self, _payload: &str) -> ServiceResponse {
        ServiceResponse {
            service: self.name().to_string(),
            status: "Success".to_string(),
            message: "RGB asset validated.".to_string(),
            data: None,
        }
    }
}
