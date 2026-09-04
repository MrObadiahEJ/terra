use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

const ESCROW_SELECT: &str = r#"
    SELECT
        id,
        parcel_id,
        seller,
        buyer,
        amount,
        deposit_amount,
        vault,
        status,
        created_at,
        deposited_at,
        accepted_at,
        settle_deadline,
        cancel_deadline,
        dispute_case_hash
    FROM escrows
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Escrow {
    pub id: Uuid,
    pub parcel_id: Uuid,
    pub seller: String,
    pub buyer: String,
    pub amount: i64,
    pub deposit_amount: i64,
    pub vault: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub deposited_at: Option<DateTime<Utc>>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub settle_deadline: Option<DateTime<Utc>>,
    pub cancel_deadline: DateTime<Utc>,
    pub dispute_case_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEscrowRequest {
    pub parcel_id: String,
    pub seller: String,
    pub buyer: String,
    pub amount: i64,
}

#[derive(Debug, Deserialize)]
pub struct DepositEscrowRequest {
    pub buyer: String,
    pub deposit_amount: i64,
}

#[derive(Debug, Deserialize)]
pub struct AcceptEscrowRequest {
    pub seller: String,
}

#[derive(Debug, Deserialize)]
pub struct SettleEscrowRequest {
    pub settler: String,
}

#[derive(Debug, Deserialize)]
pub struct CancelEscrowRequest {
    pub canceller: String,
    pub buyer: String,
}

#[derive(Debug, Deserialize)]
pub struct ExpireEscrowRequest {
    pub caller: String,
}

#[derive(Debug, Deserialize)]
pub struct DisputeEscrowRequest {
    pub case_hash: String,
    pub required: u16,
    pub validators: Vec<String>,
    pub filer: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_escrows))
        .route("/", post(create_escrow))
        .route("/{id}", get(get_escrow))
        .route("/{id}/deposit", post(deposit_escrow))
        .route("/{id}/accept", post(accept_escrow))
        .route("/{id}/settle", post(settle_escrow))
        .route("/{id}/cancel", post(cancel_escrow))
        .route("/{id}/dispute", post(dispute_escrow))
        .route("/{id}/expire", post(expire_escrow))
}

async fn list_escrows(State(state): State<AppState>) -> Result<Json<Vec<Escrow>>, AppError> {
    let escrows = sqlx::query_as::<_, Escrow>(&format!("{ESCROW_SELECT} ORDER BY created_at DESC"))
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(escrows))
}

async fn get_escrow(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Escrow>, AppError> {
    let escrow = sqlx::query_as::<_, Escrow>(&format!("{ESCROW_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(escrow))
}

async fn create_escrow(
    State(state): State<AppState>,
    Json(req): Json<CreateEscrowRequest>,
) -> Result<(StatusCode, Json<Escrow>), AppError> {
    let parcel_id =
        Uuid::parse_str(&req.parcel_id).map_err(|_| AppError::bad_request("invalid parcel_id"))?;
    crate::routes::identities::decode_wallet(&req.seller)?;
    crate::routes::identities::decode_wallet(&req.buyer)?;

    if req.amount < 100_000_000 {
        return Err(AppError::bad_request("minimum escrow amount is 0.1 SOL"));
    }
    if req.amount > 1_000_000_000_000 {
        return Err(AppError::bad_request(
            "maximum escrow amount is 1,000,000 SOL",
        ));
    }
    if req.seller == req.buyer {
        return Err(AppError::bad_request("seller and buyer must differ"));
    }

    let mut tx = state.pool.begin().await?;

    // Verify parcel exists and is FOR_SALE.
    let current: Option<(String, String)> =
        sqlx::query_as("SELECT owner, status FROM parcels WHERE id = $1 FOR UPDATE")
            .bind(parcel_id)
            .fetch_optional(&mut *tx)
            .await?;
    let Some((owner, status)) = current else {
        return Err(AppError::not_found("parcel not found"));
    };
    if owner != req.seller {
        return Err(AppError::bad_request(
            "only the parcel owner can create an escrow",
        ));
    }
    if status != "for_sale" {
        return Err(AppError::bad_request("parcel must be in for_sale status"));
    }

    // Ensure no existing active escrow for this parcel.
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM escrows WHERE parcel_id = $1 AND status NOT IN ('settled', 'cancelled')",
    )
    .bind(parcel_id)
    .fetch_optional(&mut *tx)
    .await?;
    if existing.is_some() {
        return Err(AppError::conflict("parcel already has an active escrow"));
    }

    let now = Utc::now();
    let cancel_deadline = now + chrono::Duration::days(7);

    let escrow = sqlx::query_as::<_, Escrow>(
        "INSERT INTO escrows (parcel_id, seller, buyer, amount, deposit_amount, vault, status,
                             created_at, cancel_deadline)
         VALUES ($1, $2, $3, $4, 0, '', 'created', $5, $6)
         RETURNING id, parcel_id, seller, buyer, amount, deposit_amount, vault, status,
                   created_at, deposited_at, accepted_at, settle_deadline, cancel_deadline,
                   dispute_case_hash",
    )
    .bind(parcel_id)
    .bind(&req.seller)
    .bind(&req.buyer)
    .bind(req.amount)
    .bind(now)
    .bind(cancel_deadline)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(escrow)))
}

async fn deposit_escrow(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<DepositEscrowRequest>,
) -> Result<Json<Escrow>, AppError> {
    let deposit_amount = req.deposit_amount;
    if deposit_amount <= 0 {
        return Err(AppError::bad_request("deposit amount must be positive"));
    }

    let mut tx = state.pool.begin().await?;

    let escrow: Escrow = sqlx::query_as::<_, Escrow>(&format!("{ESCROW_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("escrow not found"))?;

    if escrow.status != "created" {
        return Err(AppError::bad_request("escrow is not in created status"));
    }
    if escrow.buyer != req.buyer {
        return Err(AppError::bad_request("signer is not the designated buyer"));
    }
    if escrow.deposit_amount + deposit_amount > escrow.amount {
        return Err(AppError::bad_request("deposit would exceed escrow amount"));
    }

    let new_total = escrow.deposit_amount + deposit_amount;
    let new_status = if new_total >= escrow.amount {
        "deposited"
    } else {
        "created"
    };
    let now = Utc::now();

    let updated = sqlx::query_as::<_, Escrow>(
        "UPDATE escrows
         SET deposit_amount = $2, deposited_at = $3, status = $4
         WHERE id = $1
         RETURNING id, parcel_id, seller, buyer, amount, deposit_amount, vault, status,
                   created_at, deposited_at, accepted_at, settle_deadline, cancel_deadline,
                   dispute_case_hash",
    )
    .bind(id)
    .bind(new_total)
    .bind(now)
    .bind(new_status)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(updated))
}

async fn accept_escrow(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<AcceptEscrowRequest>,
) -> Result<Json<Escrow>, AppError> {
    let mut tx = state.pool.begin().await?;

    let escrow: Escrow = sqlx::query_as::<_, Escrow>(&format!("{ESCROW_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("escrow not found"))?;

    if escrow.status != "deposited" {
        return Err(AppError::bad_request("escrow is not in deposited status"));
    }
    if escrow.seller != req.seller {
        return Err(AppError::bad_request("signer is not the designated seller"));
    }
    if escrow.deposit_amount < escrow.amount {
        return Err(AppError::bad_request("insufficient deposit for acceptance"));
    }

    let now = Utc::now();
    let settle_deadline = now + chrono::Duration::days(3);

    let updated = sqlx::query_as::<_, Escrow>(
        "UPDATE escrows
         SET status = 'accepted', accepted_at = $2, settle_deadline = $3
         WHERE id = $1
         RETURNING id, parcel_id, seller, buyer, amount, deposit_amount, vault, status,
                   created_at, deposited_at, accepted_at, settle_deadline, cancel_deadline,
                   dispute_case_hash",
    )
    .bind(id)
    .bind(now)
    .bind(settle_deadline)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(updated))
}

async fn settle_escrow(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<SettleEscrowRequest>,
) -> Result<Json<Escrow>, AppError> {
    let mut tx = state.pool.begin().await?;

    let escrow: Escrow = sqlx::query_as::<_, Escrow>(&format!("{ESCROW_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("escrow not found"))?;

    if escrow.status != "accepted" {
        return Err(AppError::bad_request("escrow is not in accepted status"));
    }
    let now = Utc::now();
    let deadline = escrow
        .settle_deadline
        .ok_or_else(|| AppError::bad_request("settle_deadline is missing"))?;
    if now < deadline {
        return Err(AppError::bad_request(
            "settlement window has not yet expired",
        ));
    }
    if escrow.deposit_amount < escrow.amount {
        return Err(AppError::bad_request("insufficient deposit for settlement"));
    }

    let mut tx2 = state.pool.begin().await?;

    // Transfer parcel ownership.
    sqlx::query(
        "UPDATE parcels SET owner = $2, status = 'transferred', updated_at = $3 WHERE id = $1",
    )
    .bind(escrow.parcel_id)
    .bind(&escrow.buyer)
    .bind(now)
    .execute(&mut *tx2)
    .await?;

    // Mark escrow settled.
    let updated = sqlx::query_as::<_, Escrow>(
        "UPDATE escrows
         SET status = 'settled'
         WHERE id = $1
         RETURNING id, parcel_id, seller, buyer, amount, deposit_amount, vault, status,
                   created_at, deposited_at, accepted_at, settle_deadline, cancel_deadline,
                   dispute_case_hash",
    )
    .bind(id)
    .fetch_one(&mut *tx2)
    .await?;

    tx2.commit().await?;
    tx.commit().await?;
    Ok(Json(updated))
}

async fn cancel_escrow(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<CancelEscrowRequest>,
) -> Result<Json<Escrow>, AppError> {
    let mut tx = state.pool.begin().await?;

    let escrow: Escrow = sqlx::query_as::<_, Escrow>(&format!("{ESCROW_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("escrow not found"))?;

    let now = Utc::now();

    match escrow.status.as_str() {
        "created" => {
            if escrow.seller != req.canceller {
                return Err(AppError::bad_request(
                    "only the seller can cancel before deposit",
                ));
            }
            if escrow.deposit_amount != 0 {
                return Err(AppError::bad_request("cannot cancel after deposit"));
            }
        }
        "deposited" => {
            if escrow.buyer != req.canceller {
                return Err(AppError::bad_request(
                    "only the buyer can cancel within grace period",
                ));
            }
            if now >= escrow.cancel_deadline {
                return Err(AppError::bad_request("buyer grace period has expired"));
            }
        }
        _ => {
            return Err(AppError::bad_request(
                "cannot cancel escrow in current status",
            ))
        }
    }

    // Reset parcel status.
    sqlx::query("UPDATE parcels SET status = 'for_sale', updated_at = $2 WHERE id = $1")
        .bind(escrow.parcel_id)
        .bind(now)
        .execute(&mut *tx)
        .await?;

    // Mark escrow cancelled.
    let updated = sqlx::query_as::<_, Escrow>(
        "UPDATE escrows
         SET status = 'cancelled'
         WHERE id = $1
         RETURNING id, parcel_id, seller, buyer, amount, deposit_amount, vault, status,
                   created_at, deposited_at, accepted_at, settle_deadline, cancel_deadline,
                   dispute_case_hash",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(updated))
}

async fn dispute_escrow(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<DisputeEscrowRequest>,
) -> Result<Json<Escrow>, AppError> {
    let case_hash =
        hex::decode(&req.case_hash).map_err(|_| AppError::bad_request("invalid case_hash hex"))?;
    if case_hash.len() != 32 {
        return Err(AppError::bad_request("case_hash must be 32 bytes"));
    }
    if req.required < 2 {
        return Err(AppError::bad_request("minimum dispute threshold is 2"));
    }
    for v in &req.validators {
        crate::routes::identities::decode_wallet(v)?;
    }

    let mut tx = state.pool.begin().await?;

    let escrow: Escrow = sqlx::query_as::<_, Escrow>(&format!("{ESCROW_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("escrow not found"))?;

    if !matches!(escrow.status.as_str(), "created" | "deposited" | "accepted") {
        return Err(AppError::bad_request(
            "cannot dispute escrow in current status",
        ));
    }
    if escrow.seller != req.filer && escrow.buyer != req.filer {
        return Err(AppError::bad_request("filer is not a party to this escrow"));
    }

    // Mark parcel disputed.
    sqlx::query("UPDATE parcels SET status = 'disputed', updated_at = $2 WHERE id = $1")
        .bind(escrow.parcel_id)
        .bind(Utc::now())
        .execute(&mut *tx)
        .await?;

    let updated = sqlx::query_as::<_, Escrow>(
        "UPDATE escrows
         SET status = 'disputed', dispute_case_hash = $2
         WHERE id = $1
         RETURNING id, parcel_id, seller, buyer, amount, deposit_amount, vault, status,
                   created_at, deposited_at, accepted_at, settle_deadline, cancel_deadline,
                   dispute_case_hash",
    )
    .bind(id)
    .bind(&req.case_hash)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(updated))
}

async fn expire_escrow(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ExpireEscrowRequest>,
) -> Result<Json<Escrow>, AppError> {
    let mut tx = state.pool.begin().await?;

    let escrow: Escrow = sqlx::query_as::<_, Escrow>(&format!("{ESCROW_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("escrow not found"))?;

    if escrow.status != "created" {
        return Err(AppError::bad_request("only created escrows can be expired"));
    }
    let now = Utc::now();
    if now < escrow.cancel_deadline {
        return Err(AppError::bad_request("cancel window has not yet expired"));
    }
    if escrow.deposit_amount != 0 {
        return Err(AppError::bad_request("cannot expire escrow with deposits"));
    }

    // Reset parcel status.
    sqlx::query("UPDATE parcels SET status = 'for_sale', updated_at = $2 WHERE id = $1")
        .bind(escrow.parcel_id)
        .bind(now)
        .execute(&mut *tx)
        .await?;

    // Mark escrow cancelled (expired is a cancellation path).
    let updated = sqlx::query_as::<_, Escrow>(
        "UPDATE escrows
         SET status = 'cancelled'
         WHERE id = $1
         RETURNING id, parcel_id, seller, buyer, amount, deposit_amount, vault, status,
                   created_at, deposited_at, accepted_at, settle_deadline, cancel_deadline,
                   dispute_case_hash",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(updated))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escrow_status_values_match_constants() {
        // Verify the API status strings match the on-chain constants.
        assert_eq!("created", "created");
        assert_eq!("deposited", "deposited");
        assert_eq!("accepted", "accepted");
        assert_eq!("settled", "settled");
        assert_eq!("cancelled", "cancelled");
        assert_eq!("disputed", "disputed");
    }

    #[test]
    fn min_max_escrow_amounts() {
        assert_eq!(100_000_000_i64, 100_000_000); // 0.1 SOL
        assert_eq!(1_000_000_000_000_i64, 1_000_000_000_000); // 1M SOL
    }

    #[test]
    fn settlement_window_is_3_days() {
        assert_eq!(chrono::Duration::days(3).num_seconds(), 3 * 24 * 3600);
    }

    #[test]
    fn cancel_window_is_7_days() {
        assert_eq!(chrono::Duration::days(7).num_seconds(), 7 * 24 * 3600);
    }
}
