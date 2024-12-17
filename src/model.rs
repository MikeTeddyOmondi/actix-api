use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct CheckoutInfo {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub amount: String,
}

#[derive(Serialize, Clone)]
pub struct InitCheckoutOkResponse {
    pub message: &'static str,
    pub data: intasend::CheckoutResponse,
}

#[derive(Serialize, Clone)]
pub struct InitCheckoutBadRequestResponse {
    pub message: &'static str,
}

#[derive(Serialize)] // Ensure the enum is serializable
pub enum InitCheckoutResponse {
    Success(InitCheckoutOkResponse),
    BadRequest(InitCheckoutBadRequestResponse),
}

// Implement Responder for CheckoutResponse
impl actix_web::Responder for InitCheckoutResponse {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        match self {
            InitCheckoutResponse::Success(data) => {
                let body = serde_json::to_string(&data).unwrap();
                // Create response and set content type
                actix_web::HttpResponse::Ok()
                    .content_type(actix_web::http::header::ContentType::json())
                    .body(body)
            }
            InitCheckoutResponse::BadRequest(data) => {
                let body = serde_json::to_string(&data).unwrap();
                // Create response and set content type
                actix_web::HttpResponse::BadRequest()
                    .content_type(actix_web::http::header::ContentType::json())
                    .body(body)
            }
        }
    }
}

