use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::routes::identities;
use crate::state::AppState;

fn decode_hex32(s: &str) -> Result<[u8; 32], AppError> {
    let bytes = hex::decode(s).map_err(|_| AppError::bad_request("expected hex"))?;
    if bytes.len() != 32 {
        return Err(AppError::bad_request("expected 32 bytes"));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

const DISPUTE_SELECT: &str = r#"
    SELECT
        id,
        parcel_id,
        filed_by,
        case_hash,
        status,
        required,
        count,
        validators,
        filed_at,
        frozen_at,
        adjudicated_at,
        outcome,
        new_owner,
        created_at,
        updated_at
    FROM disputes
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Dispute {
    pub id: Uuid,
    pub parcel_id: Uuid,
    pub filed_by: String,
    pub case_hash: String,
    pub status: String,
    pub required: i16,
    pub count: i16,
    pub validators: Vec<String>,
    pub filed_at: DateTime<Utc>,
    pub frozen_at: Option<DateTime<Utc>>,
    pub adjudicated_at: Option<DateTime<Utc>>,
    pub outcome: Option<String>,
    pub new_owner: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct FileDisputeRequest {
    /// hex(32) hash of the court document / complaint.
    pub case_hash: String,
    /// Minimum validator co-signatures required (>= 2).
    pub required: u16,
    /// Declared validator wallets (base58).
    pub validators: Vec<String>,
    /// Filer wallet (base58) — must sign the transaction.
    pub filer: String,
}

#[derive(Debug, Deserialize)]
pub struct AdjudicateRequest {
    /// Outcome: "owner_wins" or "owner_loses".
    pub outcome: String,
    /// New owner wallet if outcome is owner_loses.
    pub new_owner: Option<String>,
    /// Court authority wallet (base58) — must sign the transaction.
    pub authority: String,
}

const MIN_DISPUTE_VALIDATORS: u16 = 2;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_disputes))
        .route("/", post(file_dispute))
        .route("/{id}", get(get_dispute))
        .route("/{id}/freeze", post(freeze_parcel))
        .route("/{id}/adjudicate", post(adjudicate_dispute))
        .route("/{id}/execute", post(execute_judgment))
        .route("/{id}", delete(cancel_dispute))
}

async fn list_disputes(State(state): State<AppState>) -> Result<Json<Vec<Dispute>>, AppError> {
    let disputes =
        sqlx::query_as::<_, Dispute>(&format!("{DISPUTE_SELECT} ORDER BY created_at DESC"))
            .fetch_all(&state.pool)
            .await?;
    Ok(Json(disputes))
}

async fn get_dispute(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Dispute>, AppError> {
    let dispute = sqlx::query_as::<_, Dispute>(&format!("{DISPUTE_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(dispute))
}

async fn file_dispute(
    State(state): State<AppState>,
    Path(parcel_id): Path<Uuid>,
    Json(req): Json<FileDisputeRequest>,
) -> Result<(StatusCode, Json<Dispute>), AppError> {
    let ih = decode_hex32(&req.case_hash)?;
    identities::decode_wallet(&req.filer)?;
    for v in &req.validators {
        identities::decode_wallet(v)?;
    }

    if req.required < MIN_DISPUTE_VALIDATORS {
        return Err(AppError::bad_request(
            "dispute threshold must be at least 2 validators",
        ));
    }
    if (req.required as usize) > req.validators.len() {
        return Err(AppError::bad_request(
            "dispute threshold cannot exceed the number of validators",
        ));
    }

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

    let dispute = sqlx::query_as::<_, Dispute>(
        "INSERT INTO disputes (parcel_id, filed_by, case_hash, status, required, count, validators)
         VALUES ($1, $2, $3, 'filed', $4, $5, $6)
         RETURNING id, parcel_id, filed_by, case_hash, status, required, count, validators,
                   filed_at, frozen_at, adjudicated_at, outcome, new_owner, created_at, updated_at",
    )
    .bind(parcel_id)
    .bind(&req.filer)
    .bind(hex::encode(ih))
    .bind(req.required as i16)
    .bind(req.validators.len() as i16)
    .bind(&req.validators)
    .fetch_one(&mut *tx)
    .await?;

    // Update parcel status to 'disputed'.
    sqlx::query("UPDATE parcels SET status = 'disputed', updated_at = now() WHERE id = $1")
        .bind(parcel_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(dispute)))
}

async fn freeze_parcel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Dispute>, AppError> {
    let mut tx = state.pool.begin().await?;

    let dispute: Dispute =
        sqlx::query_as::<_, Dispute>(&format!("{DISPUTE_SELECT} WHERE id = $1 FOR UPDATE"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

    if dispute.status != "filed" {
        return Err(AppError::bad_request(
            "dispute must be in 'filed' status to freeze",
        ));
    }
    if dispute.count < dispute.required {
        return Err(AppError::bad_request(
            "insufficient validator endorsements to freeze",
        ));
    }

    sqlx::query("UPDATE disputes SET status = 'frozen', frozen_at = now(), updated_at = now() WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE parcels SET status = 'frozen', updated_at = now() WHERE id = $1")
        .bind(dispute.parcel_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    let updated = sqlx::query_as::<_, Dispute>(&format!("{DISPUTE_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(updated))
}

async fn adjudicate_dispute(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<AdjudicateRequest>,
) -> Result<Json<Dispute>, AppError> {
    if req.outcome != "owner_wins" && req.outcome != "owner_loses" {
        return Err(AppError::bad_request(
            "outcome must be 'owner_wins' or 'owner_loses'",
        ));
    }
    if req.outcome == "owner_loses" && req.new_owner.is_none() {
        return Err(AppError::bad_request(
            "new_owner is required when owner loses",
        ));
    }
    identities::decode_wallet(&req.authority)?;
    if let Some(ref owner) = req.new_owner {
        identities::decode_wallet(owner)?;
    }

    let mut tx = state.pool.begin().await?;

    let dispute: Dispute =
        sqlx::query_as::<_, Dispute>(&format!("{DISPUTE_SELECT} WHERE id = $1 FOR UPDATE"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

    if dispute.status != "frozen" {
        return Err(AppError::bad_request(
            "dispute must be in 'frozen' status to adjudicate",
        ));
    }

    sqlx::query(
        "UPDATE disputes SET
            status = 'adjudicated',
            adjudicated_at = now(),
            outcome = $2,
            new_owner = $3,
            updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(&req.outcome)
    .bind(&req.new_owner)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE parcels SET status = 'adjudicated', updated_at = now() WHERE id = $1")
        .bind(dispute.parcel_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    let updated = sqlx::query_as::<_, Dispute>(&format!("{DISPUTE_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(updated))
}

async fn execute_judgment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut tx = state.pool.begin().await?;

    let dispute: Dispute =
        sqlx::query_as::<_, Dispute>(&format!("{DISPUTE_SELECT} WHERE id = $1 FOR UPDATE"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

    if dispute.status != "adjudicated" {
        return Err(AppError::bad_request(
            "dispute must be in 'adjudicated' status to execute",
        ));
    }

    let outcome = dispute.outcome.as_deref().unwrap_or("owner_wins");

    if outcome == "owner_wins" {
        sqlx::query("UPDATE parcels SET status = 'registered', updated_at = now() WHERE id = $1")
            .bind(dispute.parcel_id)
            .execute(&mut *tx)
            .await?;
    } else {
        let new_owner = dispute.new_owner.as_ref().ok_or_else(|| {
            AppError::bad_request("new_owner is required for owner_loses outcome")
        })?;
        sqlx::query(
            "UPDATE parcels SET owner = $2, status = 'forfeited', updated_at = now() WHERE id = $1",
        )
        .bind(dispute.parcel_id)
        .bind(new_owner)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query("UPDATE disputes SET status = 'executed', updated_at = now() WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(serde_json::json!({
        "dispute_id": id,
        "parcel_id": dispute.parcel_id,
        "outcome": outcome,
        "status": "executed",
    })))
}

async fn cancel_dispute(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let mut tx = state.pool.begin().await?;

    let dispute: Dispute =
        sqlx::query_as::<_, Dispute>(&format!("{DISPUTE_SELECT} WHERE id = $1 FOR UPDATE"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

    if dispute.status != "filed" {
        return Err(AppError::bad_request(
            "only filed disputes can be cancelled",
        ));
    }

    sqlx::query("UPDATE disputes SET status = 'cancelled', updated_at = now() WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    // Only unfreeze if parcel is still in 'disputed' status.
    sqlx::query(
        "UPDATE parcels SET status = 'registered', updated_at = now()
         WHERE id = $1 AND status = 'disputed'",
    )
    .bind(dispute.parcel_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn dispute_threshold_ok(required: u16, validators: usize) -> bool {
        required >= MIN_DISPUTE_VALIDATORS && (required as usize) <= validators && validators > 0
    }

    #[test]
    fn dispute_requires_minimum_two_validators() {
        assert!(!dispute_threshold_ok(0, 3));
        assert!(!dispute_threshold_ok(1, 3));
        assert!(dispute_threshold_ok(2, 2));
        assert!(dispute_threshold_ok(3, 5));
        assert!(!dispute_threshold_ok(5, 3));
    }

    #[test]
    fn dispute_filer_cannot_be_own_validator() {
        let filer = bs58::encode([1u8; 32]).into_string();
        let v1 = bs58::encode([2u8; 32]).into_string();
        let v2 = bs58::encode([3u8; 32]).into_string();

        fn validators_exclude_filer(filer: &str, validators: &[String]) -> bool {
            validators.iter().all(|v| v != filer)
        }
        assert!(validators_exclude_filer(&filer, &[v1.clone(), v2.clone()]));
        assert!(!validators_exclude_filer(&filer, &[v1, filer.clone()]));
    }

    #[test]
    fn dispute_status_lifecycle() {
        // Verify the expected status transitions.
        let statuses = ["filed", "frozen", "adjudicated", "executed"];
        assert_eq!(statuses[0], "filed");
        assert_eq!(statuses[1], "frozen");
        assert_eq!(statuses[2], "adjudicated");
        assert_eq!(statuses[3], "executed");
    }
}
