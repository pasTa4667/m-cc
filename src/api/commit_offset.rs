use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{AppState, enums::AppError, types::commit_offset::CommitOffsetParams};

pub async fn commit_offset_handler(
    State(state): State<AppState>,
    Query(params): Query<CommitOffsetParams>,
) -> Response {
    let offset = params.offset;
    match state.offset_manager.append(params.into(), offset) {
        Ok(_) => (StatusCode::ACCEPTED, "Offset successfully committed").into_response(),
        Err(AppError::WriteError(msg)) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    }
}
