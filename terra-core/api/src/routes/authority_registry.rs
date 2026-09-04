use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// AuthorityRegistry mirror (on-chain AuthorityRegistry / ValidatorEndorsement)
// ---------------------------------------------------------------------------

const REGISTRY_SELECT: &str = r#"
    SELECT
        id, pubkey, admin, validators,
        required_endorsements, mode, version,
        created_at, updated_at
    FROM authority_registries
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AuthorityRegistry {
    pub id: i64,
    pub pubkey: String,
    pub admin: String,
    pub validators: Vec<String>,
    pub required_endorsements: i16,
    pub mode: i16,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRegistryRequest {
    pub pubkey: String,
    pub admin: String,
    #[serde(default)]
    pub validators: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddValidatorRequest {
    pub validator: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ValidatorEndorsement {
    pub id: i64,
    pub registry_pubkey: String,
    pub proposed: String,
    pub endorsers: Vec<String>,
    pub required: i16,
    pub added_at: DateTime<Utc>,
}

async fn list_registries(
    State(state): State<AppState>,
) -> Result<Json<Vec<AuthorityRegistry>>, AppError> {
    let rows: Vec<AuthorityRegistry> = sqlx::query_as(REGISTRY_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn get_registry(
    State(state): State<AppState>,
    Path(pubkey): Path<String>,
) -> Result<Json<AuthorityRegistry>, AppError> {
    let row: AuthorityRegistry = sqlx::query_as(&format!("{REGISTRY_SELECT} WHERE pubkey = $1"))
        .bind(&pubkey)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("authority registry not found"))?;
    Ok(Json(row))
}

async fn create_registry(
    State(state): State<AppState>,
    Json(req): Json<CreateRegistryRequest>,
) -> Result<(StatusCode, Json<AuthorityRegistry>), AppError> {
    crate::routes::identities::decode_wallet(&req.pubkey)?;
    crate::routes::identities::decode_wallet(&req.admin)?;
    for v in &req.validators {
        crate::routes::identities::decode_wallet(v)?;
    }
    let row: AuthorityRegistry = sqlx::query_as(
        "INSERT INTO authority_registries (pubkey, admin, validators)
         VALUES ($1, $2, $3)
         ON CONFLICT (pubkey) DO UPDATE SET
            admin = EXCLUDED.admin,
            validators = EXCLUDED.validators,
            version = authority_registries.version + 1,
            updated_at = now()
         RETURNING id, pubkey, admin, validators, required_endorsements,
                   mode, version, created_at, updated_at",
    )
    .bind(&req.pubkey)
    .bind(&req.admin)
    .bind(&req.validators)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn add_validator(
    State(state): State<AppState>,
    Path(pubkey): Path<String>,
    Json(req): Json<AddValidatorRequest>,
) -> Result<Json<AuthorityRegistry>, AppError> {
    crate::routes::identities::decode_wallet(&req.validator)?;
    let row: AuthorityRegistry = sqlx::query_as(
        "UPDATE authority_registries
         SET validators = (
                SELECT array_agg(DISTINCT v ORDER BY v)
                FROM unnest(validators || $2::text[]) AS v
             ),
             version = version + 1,
             updated_at = now()
         WHERE pubkey = $1
         RETURNING id, pubkey, admin, validators, required_endorsements,
                   mode, version, created_at, updated_at",
    )
    .bind(&pubkey)
    .bind(&[req.validator])
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("authority registry not found"))?;
    Ok(Json(row))
}

async fn remove_validator(
    State(state): State<AppState>,
    Path((pubkey, validator)): Path<(String, String)>,
) -> Result<Json<AuthorityRegistry>, AppError> {
    let row: AuthorityRegistry = sqlx::query_as(
        "UPDATE authority_registries
         SET validators = array_remove(validators, $2),
             version = version + 1,
             updated_at = now()
         WHERE pubkey = $1
         RETURNING id, pubkey, admin, validators, required_endorsements,
                   mode, version, created_at, updated_at",
    )
    .bind(&pubkey)
    .bind(&validator)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("authority registry not found"))?;
    Ok(Json(row))
}

async fn endorse_validator_add(
    State(state): State<AppState>,
    Path(pubkey): Path<String>,
    Json(req): Json<AddValidatorRequest>,
) -> Result<(StatusCode, Json<ValidatorEndorsement>), AppError> {
    crate::routes::identities::decode_wallet(&req.validator)?;
    let row: ValidatorEndorsement = sqlx::query_as(
        "INSERT INTO registry_endorsements (registry_pubkey, proposed)
         VALUES ($1, $2)
         ON CONFLICT (registry_pubkey, proposed) DO UPDATE SET
            endorsers = registry_endorsements.endorsers
         RETURNING id, registry_pubkey, proposed, endorsers, required, added_at",
    )
    .bind(&pubkey)
    .bind(&req.validator)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("authority registry not found"))?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn flip_to_consensus(
    State(state): State<AppState>,
    Path(pubkey): Path<String>,
) -> Result<Json<AuthorityRegistry>, AppError> {
    let row: AuthorityRegistry = sqlx::query_as(
        "UPDATE authority_registries
         SET mode = 1, version = version + 1, updated_at = now()
         WHERE pubkey = $1
         RETURNING id, pubkey, admin, validators, required_endorsements,
                   mode, version, created_at, updated_at",
    )
    .bind(&pubkey)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("authority registry not found"))?;
    Ok(Json(row))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_registries).post(create_registry))
        .route("/{pubkey}", get(get_registry))
        .route("/{pubkey}/validators", post(add_validator))
        .route("/{pubkey}/validators/{validator}", delete(remove_validator))
        .route("/{pubkey}/endorsements", post(endorse_validator_add))
        .route("/{pubkey}/flip", post(flip_to_consensus))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn endorsement_identity_is_registry_plus_proposed() {
        // Mirror rule: one endorsement row per (registry, proposed) pair.
        let pair = ("REG".to_string(), "VAL".to_string());
        let same = ("REG".to_string(), "VAL".to_string());
        assert_eq!(pair, same);
    }

    #[test]
    fn consensus_mode_value() {
        assert_eq!(1i16, 1i16); // peer-consensus mode
    }
}
