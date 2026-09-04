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
// Stake pools
// ---------------------------------------------------------------------------

const STAKE_POOL_SELECT: &str = r#"
    SELECT
        stake_pool_address,
        region_registry_address,
        total_staked_lamports,
        reward_rate_bps,
        accumulated_rewards_lamports,
        last_reward_distribution,
        slash_count,
        created_at,
        updated_at
    FROM stake_pools
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct StakePool {
    pub stake_pool_address: String,
    pub region_registry_address: String,
    pub total_staked_lamports: i64,
    pub reward_rate_bps: i16,
    pub accumulated_rewards_lamports: i64,
    pub last_reward_distribution: DateTime<Utc>,
    pub slash_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStakePoolRequest {
    pub stake_pool_address: String,
    pub region_registry_address: String,
    pub reward_rate_bps: i16,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStakePoolRequest {
    pub total_staked_lamports: Option<i64>,
    pub accumulated_rewards_lamports: Option<i64>,
    pub last_reward_distribution: Option<DateTime<Utc>>,
    pub slash_count: Option<i32>,
}

// ---------------------------------------------------------------------------
// Validator stakes
// ---------------------------------------------------------------------------

const VALIDATOR_STAKE_SELECT: &str = r#"
    SELECT
        validator_stake_address,
        stake_pool_address,
        validator_address,
        staked_lamports,
        unbonding_lamports,
        unbonding_starts_at,
        rewards_accrued_lamports,
        slash_history,
        created_at,
        updated_at
    FROM validator_stakes
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ValidatorStake {
    pub validator_stake_address: String,
    pub stake_pool_address: String,
    pub validator_address: String,
    pub staked_lamports: i64,
    pub unbonding_lamports: i64,
    pub unbonding_starts_at: DateTime<Utc>,
    pub rewards_accrued_lamports: i64,
    pub slash_history: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateValidatorStakeRequest {
    pub validator_stake_address: String,
    pub stake_pool_address: String,
    pub validator_address: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateValidatorStakeRequest {
    pub staked_lamports: Option<i64>,
    pub unbonding_lamports: Option<i64>,
    pub unbonding_starts_at: Option<DateTime<Utc>>,
    pub rewards_accrued_lamports: Option<i64>,
    pub slash_history: Option<i16>,
}

// ---------------------------------------------------------------------------
// Slashing reports
// ---------------------------------------------------------------------------

const SLASHING_REPORT_SELECT: &str = r#"
    SELECT
        slashing_report_address,
        stake_pool_address,
        reporter_address,
        encode(evidence_hash, 'hex') AS evidence_hash,
        offender_address,
        offense_type,
        encode(offense_details, 'hex') AS offense_details,
        reporter_bond_lamports,
        status,
        filed_at,
        appeal_deadline,
        resolved_at,
        created_at
    FROM slashing_reports
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SlashingReport {
    pub slashing_report_address: String,
    pub stake_pool_address: String,
    pub reporter_address: String,
    pub evidence_hash: String,
    pub offender_address: String,
    pub offense_type: i16,
    pub offense_details: String,
    pub reporter_bond_lamports: i64,
    pub status: i16,
    pub filed_at: DateTime<Utc>,
    pub appeal_deadline: DateTime<Utc>,
    pub resolved_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSlashingReportRequest {
    pub slashing_report_address: String,
    pub stake_pool_address: String,
    pub reporter_address: String,
    pub evidence_hash: String,
    pub offender_address: String,
    pub offense_type: i16,
    pub offense_details: String,
    pub reporter_bond_lamports: i64,
    pub appeal_deadline: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSlashingReportRequest {
    pub status: Option<i16>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub reporter_bond_lamports: Option<i64>,
}

// ---------------------------------------------------------------------------
// Handlers — stake pools
// ---------------------------------------------------------------------------

async fn list_stake_pools(State(state): State<AppState>) -> Result<Json<Vec<StakePool>>, AppError> {
    let rows: Vec<StakePool> = sqlx::query_as(STAKE_POOL_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn get_stake_pool(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<StakePool>, AppError> {
    let row: StakePool = sqlx::query_as(&format!(
        "{STAKE_POOL_SELECT} WHERE stake_pool_address = $1"
    ))
    .bind(&address)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found(format!("stake pool {address}")))?;
    Ok(Json(row))
}

async fn create_stake_pool(
    State(state): State<AppState>,
    Json(req): Json<CreateStakePoolRequest>,
) -> Result<(StatusCode, Json<StakePool>), AppError> {
    let row: StakePool = sqlx::query_as(&format!(
        "{STAKE_POOL_SELECT} WHERE stake_pool_address = (
            INSERT INTO stake_pools (stake_pool_address, region_registry_address, reward_rate_bps)
            VALUES ($1, $2, $3)
            RETURNING stake_pool_address
        )"
    ))
    .bind(&req.stake_pool_address)
    .bind(&req.region_registry_address)
    .bind(req.reward_rate_bps)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn update_stake_pool(
    State(state): State<AppState>,
    Path(address): Path<String>,
    Json(req): Json<UpdateStakePoolRequest>,
) -> Result<Json<StakePool>, AppError> {
    sqlx::query(
        "UPDATE stake_pools SET
            total_staked_lamports = COALESCE($2, total_staked_lamports),
            accumulated_rewards_lamports = COALESCE($3, accumulated_rewards_lamports),
            last_reward_distribution = COALESCE($4, last_reward_distribution),
            slash_count = COALESCE($5, slash_count),
            updated_at = NOW()
        WHERE stake_pool_address = $1",
    )
    .bind(&address)
    .bind(req.total_staked_lamports)
    .bind(req.accumulated_rewards_lamports)
    .bind(req.last_reward_distribution)
    .bind(req.slash_count)
    .execute(&state.pool)
    .await?;

    let row: StakePool = sqlx::query_as(&format!(
        "{STAKE_POOL_SELECT} WHERE stake_pool_address = $1"
    ))
    .bind(&address)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found(format!("stake pool {address}")))?;
    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// Handlers — validator stakes
// ---------------------------------------------------------------------------

async fn list_validator_stakes(
    State(state): State<AppState>,
) -> Result<Json<Vec<ValidatorStake>>, AppError> {
    let rows: Vec<ValidatorStake> = sqlx::query_as(VALIDATOR_STAKE_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn get_validator_stake(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<ValidatorStake>, AppError> {
    let row: ValidatorStake = sqlx::query_as(&format!(
        "{VALIDATOR_STAKE_SELECT} WHERE validator_stake_address = $1"
    ))
    .bind(&address)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found(format!("validator stake {address}")))?;
    Ok(Json(row))
}

async fn create_validator_stake(
    State(state): State<AppState>,
    Json(req): Json<CreateValidatorStakeRequest>,
) -> Result<(StatusCode, Json<ValidatorStake>), AppError> {
    let row: ValidatorStake = sqlx::query_as(&format!(
        "{VALIDATOR_STAKE_SELECT} WHERE validator_stake_address = (
            INSERT INTO validator_stakes (validator_stake_address, stake_pool_address, validator_address)
            VALUES ($1, $2, $3)
            RETURNING validator_stake_address
        )"
    ))
    .bind(&req.validator_stake_address)
    .bind(&req.stake_pool_address)
    .bind(&req.validator_address)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn update_validator_stake(
    State(state): State<AppState>,
    Path(address): Path<String>,
    Json(req): Json<UpdateValidatorStakeRequest>,
) -> Result<Json<ValidatorStake>, AppError> {
    sqlx::query(
        "UPDATE validator_stakes SET
            staked_lamports = COALESCE($2, staked_lamports),
            unbonding_lamports = COALESCE($3, unbonding_lamports),
            unbonding_starts_at = COALESCE($4, unbonding_starts_at),
            rewards_accrued_lamports = COALESCE($5, rewards_accrued_lamports),
            slash_history = COALESCE($6, slash_history),
            updated_at = NOW()
        WHERE validator_stake_address = $1",
    )
    .bind(&address)
    .bind(req.staked_lamports)
    .bind(req.unbonding_lamports)
    .bind(req.unbonding_starts_at)
    .bind(req.rewards_accrued_lamports)
    .bind(req.slash_history)
    .execute(&state.pool)
    .await?;

    let row: ValidatorStake = sqlx::query_as(&format!(
        "{VALIDATOR_STAKE_SELECT} WHERE validator_stake_address = $1"
    ))
    .bind(&address)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found(format!("validator stake {address}")))?;
    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// Handlers — slashing reports
// ---------------------------------------------------------------------------

async fn list_slashing_reports(
    State(state): State<AppState>,
) -> Result<Json<Vec<SlashingReport>>, AppError> {
    let rows: Vec<SlashingReport> = sqlx::query_as(SLASHING_REPORT_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn get_slashing_report(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<SlashingReport>, AppError> {
    let row: SlashingReport = sqlx::query_as(&format!(
        "{SLASHING_REPORT_SELECT} WHERE slashing_report_address = $1"
    ))
    .bind(&address)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found(format!("slashing report {address}")))?;
    Ok(Json(row))
}

async fn create_slashing_report(
    State(state): State<AppState>,
    Json(req): Json<CreateSlashingReportRequest>,
) -> Result<(StatusCode, Json<SlashingReport>), AppError> {
    let row: SlashingReport = sqlx::query_as(&format!(
        "{SLASHING_REPORT_SELECT} WHERE slashing_report_address = (
            INSERT INTO slashing_reports (
                slashing_report_address, stake_pool_address, reporter_address,
                evidence_hash, offender_address, offense_type, offense_details,
                reporter_bond_lamports, appeal_deadline
            )
            VALUES ($1, $2, $3, decode($4, 'hex'), $5, $6, decode($7, 'hex'), $8, $9)
            RETURNING slashing_report_address
        )"
    ))
    .bind(&req.slashing_report_address)
    .bind(&req.stake_pool_address)
    .bind(&req.reporter_address)
    .bind(&req.evidence_hash)
    .bind(&req.offender_address)
    .bind(req.offense_type)
    .bind(&req.offense_details)
    .bind(req.reporter_bond_lamports)
    .bind(req.appeal_deadline)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn update_slashing_report(
    State(state): State<AppState>,
    Path(address): Path<String>,
    Json(req): Json<UpdateSlashingReportRequest>,
) -> Result<Json<SlashingReport>, AppError> {
    sqlx::query(
        "UPDATE slashing_reports SET
            status = COALESCE($2, status),
            resolved_at = COALESCE($3, resolved_at),
            reporter_bond_lamports = COALESCE($4, reporter_bond_lamports)
        WHERE slashing_report_address = $1",
    )
    .bind(&address)
    .bind(req.status)
    .bind(req.resolved_at)
    .bind(req.reporter_bond_lamports)
    .execute(&state.pool)
    .await?;

    let row: SlashingReport = sqlx::query_as(&format!(
        "{SLASHING_REPORT_SELECT} WHERE slashing_report_address = $1"
    ))
    .bind(&address)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found(format!("slashing report {address}")))?;
    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        // Stake pools
        .route("/pools", get(list_stake_pools).post(create_stake_pool))
        .route(
            "/pools/{address}",
            get(get_stake_pool).put(update_stake_pool),
        )
        // Validator stakes
        .route(
            "/stakes",
            get(list_validator_stakes).post(create_validator_stake),
        )
        .route(
            "/stakes/{address}",
            get(get_validator_stake).put(update_validator_stake),
        )
        // Slashing reports
        .route(
            "/reports",
            get(list_slashing_reports).post(create_slashing_report),
        )
        .route(
            "/reports/{address}",
            get(get_slashing_report).put(update_slashing_report),
        )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stake_pool_struct_fields_match() {
        let _ = std::mem::size_of::<StakePool>();
    }

    #[test]
    fn validator_stake_struct_fields_match() {
        let _ = std::mem::size_of::<ValidatorStake>();
    }

    #[test]
    fn slashing_report_struct_fields_match() {
        let _ = std::mem::size_of::<SlashingReport>();
    }
}
