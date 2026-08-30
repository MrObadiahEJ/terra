use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn health(State(state): State<AppState>) -> Json<Value> {
    let db_ok = sqlx::query_scalar::<_, bool>("SELECT true")
        .fetch_one(&state.pool)
        .await
        .is_ok();

    Json(json!({
        "status": if db_ok { "ok" } else { "degraded" },
        "database": if db_ok { "up" } else { "down" },
        "osm_loaded": state.geo.is_some(),
    }))
}
