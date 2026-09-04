use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Jurisdiction
// ---------------------------------------------------------------------------

const JURISDICTION_SELECT: &str = r#"
    SELECT
        id, country_code, authority, jurisdiction_name,
        credential_schema_cid, revocation_registry,
        verification_key_hash, algorithm_id, status,
        created_at, updated_at
    FROM jurisdictions
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Jurisdiction {
    pub id: Uuid,
    pub country_code: String,
    pub authority: String,
    pub jurisdiction_name: String,
    pub credential_schema_cid: String,
    pub revocation_registry: String,
    pub verification_key_hash: String,
    pub algorithm_id: i16,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterJurisdictionRequest {
    pub country_code: String,
    pub authority: String,
    pub jurisdiction_name: String,
    pub credential_schema_cid: String,
    pub revocation_registry: String,
    pub verification_key_hash: String,
    pub algorithm_id: u8,
}

#[derive(Debug, Deserialize)]
pub struct UpdateJurisdictionRequest {
    pub verification_key_hash: Option<String>,
    pub revocation_registry: Option<String>,
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// Cross-border binding
// ---------------------------------------------------------------------------

const BINDING_SELECT: &str = r#"
    SELECT
        b.id, b.jurisdiction_id, b.identity_hash,
        b.credential_commitment, b.nullifier,
        b.proof_data, b.proof_version, b.algorithm_id,
        b.revoked, b.revoked_at, b.revoked_by,
        b.bound_at, b.expires_at, b.version,
        j.country_code
    FROM cross_border_bindings b
    JOIN jurisdictions j ON j.id = b.jurisdiction_id
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Binding {
    pub id: Uuid,
    pub jurisdiction_id: Uuid,
    pub identity_hash: String,
    pub credential_commitment: String,
    pub nullifier: String,
    pub proof_data: String,
    pub proof_version: i16,
    pub algorithm_id: i16,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_by: Option<String>,
    pub bound_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub version: i64,
    pub country_code: String,
}

#[derive(Debug, Deserialize)]
pub struct BindIdentityRequest {
    pub jurisdiction_id: Uuid,
    pub identity_hash: String,
    pub credential_commitment: String,
    pub proof_data: String,
    pub nullifier_nonce: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RevokeBindingRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct RebindRequest {
    pub credential_commitment: String,
    pub proof_data: String,
    pub nullifier_nonce: String,
    pub expires_at: Option<String>,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/jurisdictions",
            get(list_jurisdictions).post(register_jurisdiction),
        )
        .route(
            "/jurisdictions/{id}",
            get(get_jurisdiction).patch(update_jurisdiction),
        )
        .route("/bindings", get(list_bindings).post(bind_identity))
        .route("/bindings/{id}", get(get_binding))
        .route("/bindings/{id}/verify", post(verify_membership))
        .route("/bindings/{id}/revoke", post(revoke_binding))
        .route("/bindings/{id}/rebind", post(rebind_identity))
}

// ---------------------------------------------------------------------------
// Handlers — Jurisdictions
// ---------------------------------------------------------------------------

async fn list_jurisdictions(
    State(state): State<AppState>,
) -> Result<Json<Vec<Jurisdiction>>, AppError> {
    let rows: Vec<Jurisdiction> = sqlx::query_as(JURISDICTION_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn register_jurisdiction(
    State(state): State<AppState>,
    Json(req): Json<RegisterJurisdictionRequest>,
) -> Result<(StatusCode, Json<Jurisdiction>), AppError> {
    if req.country_code.is_empty() {
        return Err(AppError::bad_request("country_code is required"));
    }
    if req.jurisdiction_name.is_empty() {
        return Err(AppError::bad_request("jurisdiction_name is required"));
    }
    if req.credential_schema_cid.is_empty() {
        return Err(AppError::bad_request("credential_schema_cid is required"));
    }
    if req.verification_key_hash.is_empty() {
        return Err(AppError::bad_request("verification_key_hash is required"));
    }

    let row: Jurisdiction = sqlx::query_as(
        r#"INSERT INTO jurisdictions (
            country_code, authority, jurisdiction_name,
            credential_schema_cid, revocation_registry,
            verification_key_hash, algorithm_id
        ) VALUES ($1,$2,$3,$4,$5,$6,$7)
        RETURNING id, country_code, authority, jurisdiction_name,
            credential_schema_cid, revocation_registry,
            verification_key_hash, algorithm_id, status,
            created_at, updated_at"#,
    )
    .bind(&req.country_code)
    .bind(&req.authority)
    .bind(&req.jurisdiction_name)
    .bind(&req.credential_schema_cid)
    .bind(&req.revocation_registry)
    .bind(&req.verification_key_hash)
    .bind(req.algorithm_id as i16)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn get_jurisdiction(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Jurisdiction>, AppError> {
    let row: Jurisdiction = sqlx::query_as(&format!("{JURISDICTION_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("jurisdiction"))?;
    Ok(Json(row))
}

async fn update_jurisdiction(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateJurisdictionRequest>,
) -> Result<Json<Jurisdiction>, AppError> {
    let existing: Jurisdiction = sqlx::query_as(&format!("{JURISDICTION_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("jurisdiction"))?;

    let vk = req
        .verification_key_hash
        .unwrap_or(existing.verification_key_hash);
    let rr = req
        .revocation_registry
        .unwrap_or(existing.revocation_registry);
    let st = req.status.unwrap_or(existing.status);

    let row: Jurisdiction = sqlx::query_as(
        r#"UPDATE jurisdictions SET
            verification_key_hash = $1, revocation_registry = $2,
            status = $3, updated_at = now()
        WHERE id = $4
        RETURNING id, country_code, authority, jurisdiction_name,
            credential_schema_cid, revocation_registry,
            verification_key_hash, algorithm_id, status,
            created_at, updated_at"#,
    )
    .bind(&vk)
    .bind(&rr)
    .bind(&st)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// Handlers — Bindings
// ---------------------------------------------------------------------------

async fn list_bindings(State(state): State<AppState>) -> Result<Json<Vec<Binding>>, AppError> {
    let rows: Vec<Binding> = sqlx::query_as(BINDING_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn bind_identity(
    State(state): State<AppState>,
    Json(req): Json<BindIdentityRequest>,
) -> Result<(StatusCode, Json<Binding>), AppError> {
    if req.identity_hash.is_empty() {
        return Err(AppError::bad_request("identity_hash is required"));
    }
    if req.credential_commitment.is_empty() {
        return Err(AppError::bad_request("credential_commitment is required"));
    }
    if req.proof_data.is_empty() {
        return Err(AppError::bad_request("proof_data is required"));
    }

    let binding_id = Uuid::new_v4();
    let expires_at = parse_expires_at(req.expires_at.as_deref())?;
    sqlx::query(
        r#"INSERT INTO cross_border_bindings (
            id, jurisdiction_id, identity_hash, credential_commitment,
            nullifier, proof_data, expires_at
        ) VALUES ($1,$2,$3,$4,$5,$6,$7)"#,
    )
    .bind(binding_id)
    .bind(req.jurisdiction_id)
    .bind(&req.identity_hash)
    .bind(&req.credential_commitment)
    .bind(&req.nullifier_nonce)
    .bind(&req.proof_data)
    .bind(expires_at)
    .execute(&state.pool)
    .await?;

    let row: Binding = sqlx::query_as(&format!("{BINDING_SELECT} WHERE b.id = $1"))
        .bind(binding_id)
        .fetch_one(&state.pool)
        .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn get_binding(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Binding>, AppError> {
    let row: Binding = sqlx::query_as(&format!("{BINDING_SELECT} WHERE b.id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("binding"))?;
    Ok(Json(row))
}

async fn verify_membership(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Binding>, AppError> {
    let row: Binding = sqlx::query_as(&format!("{BINDING_SELECT} WHERE b.id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("binding"))?;

    if row.revoked {
        return Err(AppError::bad_request("binding has been revoked"));
    }
    if let Some(exp) = row.expires_at {
        if exp < Utc::now() {
            return Err(AppError::bad_request("binding has expired"));
        }
    }

    sqlx::query(r#"UPDATE cross_border_bindings SET version = version + 1 WHERE id = $1"#)
        .bind(id)
        .execute(&state.pool)
        .await?;

    let updated: Binding = sqlx::query_as(&format!("{BINDING_SELECT} WHERE b.id = $1"))
        .bind(id)
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(updated))
}

async fn revoke_binding(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<RevokeBindingRequest>,
) -> Result<Json<Binding>, AppError> {
    if req.reason.is_empty() {
        return Err(AppError::bad_request("reason is required"));
    }

    let existing: Binding = sqlx::query_as(&format!("{BINDING_SELECT} WHERE b.id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("binding"))?;

    if existing.revoked {
        return Err(AppError::bad_request("binding is already revoked"));
    }

    sqlx::query(
        r#"UPDATE cross_border_bindings SET
            revoked = true, revoked_at = now(), revoked_by = $2,
            version = version + 1
        WHERE id = $1"#,
    )
    .bind(id)
    .bind(&req.reason)
    .execute(&state.pool)
    .await?;

    let row: Binding = sqlx::query_as(&format!("{BINDING_SELECT} WHERE b.id = $1"))
        .bind(id)
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(row))
}

async fn rebind_identity(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<RebindRequest>,
) -> Result<(StatusCode, Json<Binding>), AppError> {
    let existing: Binding = sqlx::query_as(&format!("{BINDING_SELECT} WHERE b.id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("binding"))?;

    if existing.revoked {
        return Err(AppError::bad_request("revoked bindings cannot be rebound"));
    }

    // Mark old as revoked.
    sqlx::query(
        r#"UPDATE cross_border_bindings SET revoked = true, revoked_at = now() WHERE id = $1"#,
    )
    .bind(id)
    .execute(&state.pool)
    .await?;

    // Insert new binding.
    let new_id = Uuid::new_v4();
    let expires_at = parse_expires_at(req.expires_at.as_deref())?;
    sqlx::query(
        r#"INSERT INTO cross_border_bindings (
            id, jurisdiction_id, identity_hash, credential_commitment,
            nullifier, proof_data, expires_at
        ) VALUES ($1,$2,$3,$4,$5,$6,$7)"#,
    )
    .bind(new_id)
    .bind(existing.jurisdiction_id)
    .bind(&existing.identity_hash)
    .bind(&req.credential_commitment)
    .bind(&req.nullifier_nonce)
    .bind(&req.proof_data)
    .bind(expires_at)
    .execute(&state.pool)
    .await?;

    let row: Binding = sqlx::query_as(&format!("{BINDING_SELECT} WHERE b.id = $1"))
        .bind(new_id)
        .fetch_one(&state.pool)
        .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse the optional RFC-3339 expiry into a TIMESTAMPTZ bind value.
fn parse_expires_at(raw: Option<&str>) -> Result<Option<DateTime<Utc>>, AppError> {
    raw.map(|s| {
        s.parse::<DateTime<Utc>>()
            .map_err(|_| AppError::bad_request("invalid expires_at timestamp"))
    })
    .transpose()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_request_defaults() {
        let req = RegisterJurisdictionRequest {
            country_code: "KE".into(),
            authority: "".into(),
            jurisdiction_name: "".into(),
            credential_schema_cid: "".into(),
            revocation_registry: "".into(),
            verification_key_hash: "".into(),
            algorithm_id: 0,
        };
        assert_eq!(req.country_code, "KE");
        assert_eq!(req.algorithm_id, 0);
    }

    #[test]
    fn expires_at_parser() {
        assert!(parse_expires_at(None).unwrap().is_none());
        assert!(parse_expires_at(Some("2030-01-01T00:00:00Z"))
            .unwrap()
            .is_some());
        assert!(parse_expires_at(Some("not-a-date")).is_err());
    }

    #[test]
    fn bind_request_requires_fields() {
        let req = BindIdentityRequest {
            jurisdiction_id: Uuid::new_v4(),
            identity_hash: "".into(),
            credential_commitment: "".into(),
            proof_data: "".into(),
            nullifier_nonce: "".into(),
            expires_at: None,
        };
        assert!(req.identity_hash.is_empty());
        assert!(req.expires_at.is_none());
    }

    #[test]
    fn revoke_request_defaults() {
        let req = RevokeBindingRequest {
            reason: "credential revoked".into(),
        };
        assert_eq!(req.reason, "credential revoked");
    }
}
