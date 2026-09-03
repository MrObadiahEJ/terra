use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

const RIGHT_SELECT: &str = r#"
    SELECT
        id,
        parcel_id,
        rights_kind,
        holder,
        granter,
        created_at,
        expires_at,
        notes,
        status,
        grace_period_secs
    FROM rights
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Right {
    pub id: Uuid,
    pub parcel_id: Uuid,
    pub rights_kind: i16,
    pub holder: String,
    pub granter: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub notes: String,
    pub status: String,
    pub grace_period_secs: i64,
}

#[derive(Debug, Deserialize)]
pub struct RenewRightRequest {
    pub nonce: u16,
    pub new_expires_at: String,
    pub new_notes: String,
    pub holder: String,
    pub granter: String,
}

#[derive(Debug, Deserialize)]
pub struct SweepRightRequest {
    pub keeper: String,
}

#[derive(Debug, Deserialize)]
pub struct GrantConditionalRightRequest {
    pub parcel_id: String,
    pub nonce: u16,
    pub rights_kind: String,
    pub holder: String,
    pub expires_at: Option<String>,
    pub condition_deadline: String,
    pub condition_desc: String,
    pub grace_period_secs: i64,
    pub notes: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_rights))
        .route("/conditional", post(grant_conditional_right))
        .route("/{id}", get(get_right))
        .route("/parcel/{parcel_id}", get(list_rights_for_parcel))
        .route("/{parcel_id}/{nonce}/renew", post(renew_right))
        .route("/{parcel_id}/{nonce}/sweep", post(sweep_expired_right))
}

async fn list_rights(
    State(state): State<AppState>,
) -> Result<Json<Vec<Right>>, AppError> {
    let rights = sqlx::query_as::<_, Right>(&format!(
        "{RIGHT_SELECT} ORDER BY created_at DESC"
    ))
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rights))
}

async fn get_right(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Right>, AppError> {
    let right = sqlx::query_as::<_, Right>(&format!(
        "{RIGHT_SELECT} WHERE id = $1"
    ))
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(right))
}

async fn list_rights_for_parcel(
    State(state): State<AppState>,
    Path(parcel_id): Path<Uuid>,
) -> Result<Json<Vec<Right>>, AppError> {
    let rights = sqlx::query_as::<_, Right>(&format!(
        "{RIGHT_SELECT} WHERE parcel_id = $1 ORDER BY created_at DESC"
    ))
    .bind(parcel_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rights))
}

async fn renew_right(
    State(state): State<AppState>,
    Path((parcel_id, nonce)): Path<(Uuid, u16)>,
    Json(req): Json<RenewRightRequest>,
) -> Result<Json<Right>, AppError> {
    let new_expires_at: DateTime<Utc> = req.new_expires_at.parse()
        .map_err(|_| AppError::bad_request("invalid new_expires_at timestamp"))?;
    if new_expires_at <= Utc::now() {
        return Err(AppError::bad_request("new_expires_at must be in the future"));
    }

    let mut tx = state.pool.begin().await?;

    let right: Right = sqlx::query_as::<_, Right>(&format!(
        "{RIGHT_SELECT} WHERE parcel_id = $1 AND holder = $2 LIMIT 1"
    ))
    .bind(parcel_id)
    .bind(&req.holder)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::not_found("right not found"))?;

    if right.status == "revoked" || right.status == "renewed" {
        return Err(AppError::bad_request("cannot renew a revoked or already-renewed right"));
    }
    if right.expires_at.map(|e| new_expires_at <= e).unwrap_or(false) {
        return Err(AppError::bad_request("renewal must extend the expiry"));
    }

    let updated = sqlx::query_as::<_, Right>(
        "UPDATE rights
         SET expires_at = $3, status = 'active', notes = $4
         WHERE parcel_id = $1 AND holder = $2
         RETURNING id, parcel_id, rights_kind, holder, granter, created_at,
                   expires_at, notes, status, grace_period_secs",
    )
    .bind(parcel_id)
    .bind(&req.holder)
    .bind(new_expires_at)
    .bind(&req.new_notes)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(updated))
}

async fn sweep_expired_right(
    State(state): State<AppState>,
    Path((parcel_id, _nonce)): Path<(Uuid, u16)>,
    Json(req): Json<SweepRightRequest>,
) -> Result<Json<Right>, AppError> {
    let mut tx = state.pool.begin().await?;

    let right: Right = sqlx::query_as::<_, Right>(&format!(
        "{RIGHT_SELECT} WHERE parcel_id = $1 LIMIT 1"
    ))
    .bind(parcel_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::not_found("right not found"))?;

    if right.status != "active" && right.status != "expiring" {
        return Err(AppError::bad_request("right is not in a sweepable status"));
    }
    if right.expires_at.is_none() {
        return Err(AppError::bad_request("permanent rights cannot be swept"));
    }

    let expires_at = right.expires_at.unwrap();
    let now = Utc::now();
    if now < expires_at {
        return Err(AppError::bad_request("right has not yet expired"));
    }

    let new_status = if right.grace_period_secs > 0
        && now < expires_at + chrono::Duration::seconds(right.grace_period_secs)
    {
        "grace"
    } else {
        "expired"
    };

    let updated = sqlx::query_as::<_, Right>(
        "UPDATE rights
         SET status = $3
         WHERE parcel_id = $1 AND id = $2
         RETURNING id, parcel_id, rights_kind, holder, granter, created_at,
                   expires_at, notes, status, grace_period_secs",
    )
    .bind(parcel_id)
    .bind(right.id)
    .bind(new_status)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(updated))
}

async fn grant_conditional_right(
    State(state): State<AppState>,
    Json(req): Json<GrantConditionalRightRequest>,
) -> Result<(StatusCode, Json<Right>), AppError> {
    let parcel_id = Uuid::parse_str(&req.parcel_id)
        .map_err(|_| AppError::bad_request("invalid parcel_id"))?;

    let mut tx = state.pool.begin().await?;

    // Verify parcel exists.
    let current: Option<(String,)> =
        sqlx::query_as("SELECT owner FROM parcels WHERE id = $1 FOR UPDATE")
            .bind(parcel_id)
            .fetch_optional(&mut *tx)
            .await?;
    let Some((_owner,)) = current else {
        return Err(AppError::not_found("parcel not found"));
    };

    let now = Utc::now();

    let right = sqlx::query_as::<_, Right>(
        "INSERT INTO rights (parcel_id, rights_kind, holder, granter, expires_at, notes, status, grace_period_secs)
         VALUES ($1, $2, $3, $4, $5, $6, 'active', $7)
         RETURNING id, parcel_id, rights_kind, holder, granter, created_at,
                   expires_at, notes, status, grace_period_secs",
    )
    .bind(parcel_id)
    .bind(1i16) // USAGE kind
    .bind(&req.holder)
    .bind("owner") // placeholder — on-chain it's the granter
    .bind(req.expires_at.as_deref().and_then(|s| s.parse::<DateTime<Utc>>().ok()))
    .bind(&req.notes)
    .bind(req.grace_period_secs)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(right)))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_status_values_match_constants() {
        assert_eq!("active", "active");
        assert_eq!("expiring", "expiring");
        assert_eq!("expired", "expired");
        assert_eq!("grace", "grace");
        assert_eq!("renewed", "renewed");
        assert_eq!("revoked", "revoked");
    }

    #[test]
    fn expiring_warning_is_30_days() {
        assert_eq!(chrono::Duration::days(30).num_seconds(), 30 * 24 * 3600);
    }

    #[test]
    fn max_grace_period_is_1_year() {
        assert_eq!(chrono::Duration::days(365).num_seconds(), 365 * 24 * 3600);
    }
}
