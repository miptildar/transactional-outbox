use crate::model::dto::{CreateDeliveryRequest, ErrorResponseDto, CreateDeliveryResponse};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};

pub fn init_router() -> Router {
    Router::new()
        .route("/v1/delivery", post(create_delivery))
        .route("/v1/delivery/:delivery_id", get(get_delivery_by_id))
}

async fn create_delivery(
    Json(payload): Json<CreateDeliveryRequest>
) -> Result<Json<CreateDeliveryResponse>, (StatusCode, Json<ErrorResponseDto>)> {

}

async fn get_delivery_by_id(Path(delivery_id): Path<String>) -> Json<CreateDeliveryResponse> {

}

