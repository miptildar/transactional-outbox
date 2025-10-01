use crate::controller::dto::{CreateDeliveryRequest, DeliveryResponse, ErrorResponseDto};
use crate::service::delivery::{DeliveryService, ServiceError};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::sync::Arc;

pub struct AppState {
    pub delivery_service: Arc<DeliveryService>,
}

pub fn init_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/v1/delivery", post(create_delivery))
        .route("/v1/delivery/:delivery_id", get(get_delivery_by_id))
        .with_state(state)
}

async fn create_delivery(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateDeliveryRequest>,
) -> Result<Json<DeliveryResponse>, (StatusCode, Json<ErrorResponseDto>)> {
    
    let response = state.delivery_service
        .create_delivery(payload)
        .await
        .map_err(|e| create_error_response(&e))?;

    Ok(Json(response))
}

async fn get_delivery_by_id(State(state): State<Arc<AppState>>,
                            Path(delivery_id): Path<String>) -> Result<Json<DeliveryResponse>, (StatusCode, Json<ErrorResponseDto>)> {
    
    let response = state.delivery_service
        .get_delivery_by_id(&delivery_id)
        .await
        .map_err(|e| create_error_response(&e))?;
    
    Ok(Json(response))
}

fn create_error_response(error: &ServiceError) -> (StatusCode, Json<ErrorResponseDto>) {
    match error {
        ServiceError::NotFound => {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponseDto {
                    error: "Not Found",
                    message: error.to_string(),
                })
            )
        }
        
        ServiceError::InvalidDto => {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponseDto {
                    error: "Not Found",
                    message: error.to_string(),
                })
            )
        }

        ServiceError::InvalidStatusTransition => {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponseDto {
                    error: "Not Found",
                    message: error.to_string(),
                })
            )
        }
        ServiceError::DatabaseError(_) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponseDto {
                    error: "Internal Server Error",
                    message: "Database error".to_string(),
                })
            )
        }
        ServiceError::SerializationError(_) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponseDto {
                    error: "Internal Server Error",
                    message: "Serialization error".to_string(),
                })
            )
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    NotFound,
    BadRequest(String),
    InternalServerError(String),
}
