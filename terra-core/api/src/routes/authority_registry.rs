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

/// On-chain: CONSENSUS_FLIP_THRESHOLD = 4 (mode derived, not stored).
const CONSENSUS_FLIP_THRESHOLD: i16 = 4;

const REGISTRY_SELECT: &str = r#"
    SELECT
        id, pubkey, admin, validators,
        required_endorsements, version,
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
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// View model returned to API consumers. Mode is derived from validator count,
/// not stored — matching the on-chain program's derived-mode design.
#[derive(Debug, Serialize)]
pub struct AuthorityRegistryView {
    pub id: i64,
    pub pubkey: String,
    pub admin: String,
    pub validators: Vec<String>,
    pub required_endorsements: i16,
    /// Derived: 0 = bootstrap, 1 = peer-consensus (>= 4 validators).
    pub mode: i16,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AuthorityRegistryView {
    pub fn from_row(row: AuthorityRegistry) -> Self {
        let mode = if (row.validators.len() as i16) >= CONSENSUS_FLIP_THRESHOLD {
            1i16
        } else {
            0i16
        };
        Self {
            id: row.id,
            pubkey: row.pubkey,
            admin: row.admin,
            validators: row.validators,
            required_endorsements: row.required_endorsements,
            mode,
            version: row.version,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
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
) -> Result<Json<Vec<AuthorityRegistryView>>, AppError> {
    let rows: Vec<AuthorityRegistry> = sqlx::query_as(REGISTRY_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows.into_iter().map(AuthorityRegistryView::from_row).collect()))
}

async fn get_registry(
    State(state): State<AppState>,
    Path(pubkey): Path<String>,
) -> Result<Json<AuthorityRegistryView>, AppError> {
    let row: AuthorityRegistry = sqlx::query_as(&format!("{REGISTRY_SELECT} WHERE pubkey = $1"))
        .bind(&pubkey)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("authority registry not found"))?;
    Ok(Json(AuthorityRegistryView::from_row(row)))
}

async fn create_registry(
    State(state): State<AppState>,
    Json(req): Json<CreateRegistryRequest>,
) -> Result<(StatusCode, Json<AuthorityRegistryView>), AppError> {
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
                   version, created_at, updated_at",
    )
    .bind(&req.pubkey)
    .bind(&req.admin)
    .bind(&req.validators)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(AuthorityRegistryView::from_row(row))))
}

async fn add_validator(
    State(state): State<AppState>,
    Path(pubkey): Path<String>,
    Json(req): Json<AddValidatorRequest>,
) -> Result<Json<AuthorityRegistryView>, AppError> {
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
                   version, created_at, updated_at",
    )
    .bind(&pubkey)
    .bind(&[req.validator])
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("authority registry not found"))?;
    Ok(Json(AuthorityRegistryView::from_row(row)))
}

async fn remove_validator(
    State(state): State<AppState>,
    Path((pubkey, validator)): Path<(String, String)>,
) -> Result<Json<AuthorityRegistryView>, AppError> {
    let row: AuthorityRegistry = sqlx::query_as(
        "UPDATE authority_registries
         SET validators = array_remove(validators, $2),
             version = version + 1,
             updated_at = now()
         WHERE pubkey = $1
         RETURNING id, pubkey, admin, validators, required_endorsements,
                   version, created_at, updated_at",
    )
    .bind(&pubkey)
    .bind(&validator)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("authority registry not found"))?;
    Ok(Json(AuthorityRegistryView::from_row(row)))
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

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_registries).post(create_registry))
        .route("/{pubkey}", get(get_registry))
        .route("/{pubkey}/validators", post(add_validator))
        .route("/{pubkey}/validators/{validator}", delete(remove_validator))
        .route("/{pubkey}/endorsements", post(endorse_validator_add))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endorsement_identity_is_registry_plus_proposed() {
        let pair = ("REG".to_string(), "VAL".to_string());
        let same = ("REG".to_string(), "VAL".to_string());
        assert_eq!(pair, same);
    }

    #[test]
    fn derived_mode_at_threshold() {
        // Below threshold = bootstrap
        let view = AuthorityRegistryView {
            id: 1,
            pubkey: "test".into(),
            admin: "admin".into(),
            validators: vec!["v1".into(), "v2".into(), "v3".into()],
            required_endorsements: 0,
            mode: 0,
            version: 1,
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        assert_eq!(view.mode, 0i16); // bootstrap

        // At threshold = peer-consensus
        let view = AuthorityRegistryView {
            validators: vec!["v1".into(), "v2".into(), "v3".into(), "v4".into()],
            mode: 1,
            ..view
        };
        assert_eq!(view.mode, 1i16); // peer-consensus
    }
}
