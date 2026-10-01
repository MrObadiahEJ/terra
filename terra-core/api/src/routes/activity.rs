use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// B4 activity feed — recent real off-chain API activity, newest first.
//
// One UNION over the timestamped mirror tables the API actually writes to;
// the frontend merges this with demo-engine events (clearly labelled per
// source). On-chain Solana tx indexing is a separate, not-yet-built stream.
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list))
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(default)]
    pub limit: Option<i64>,
}

/// Row shape returned to the client: epoch-ms timestamps so the frontend can
/// format in one place (matches the demo store's Date.now() basis).
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ActivityItem {
    pub id: String,
    pub kind: String,
    pub summary: String,
    pub at: i64,
    pub source: String,
}

/// Same-source clamp shared by handler and tests.
fn clamp_limit(raw: Option<i64>) -> i64 {
    raw.unwrap_or(50).clamp(1, 200)
}

const FEED_SQL: &str = r#"
SELECT * FROM (
    SELECT 'parcel:' || id::text AS id, 'parcel.created' AS kind,
           'Parcel ' || left(id::text, 8) || ' registered' AS summary,
           (EXTRACT(EPOCH FROM created_at) * 1000)::bigint AS at, 'api' AS source
      FROM parcels
    UNION ALL
    SELECT 'evidence:' || id::text, 'evidence.uploaded',
           'Evidence "' || filename || '" uploaded',
           (EXTRACT(EPOCH FROM created_at) * 1000)::bigint, 'api'
      FROM evidence_uploads
    UNION ALL
    SELECT 'right:' || id::text, 'right.granted',
           'Right kind ' || rights_kind::text || ' granted to ' || left(holder, 14),
           (EXTRACT(EPOCH FROM created_at) * 1000)::bigint, 'api'
      FROM rights
    UNION ALL
    SELECT 'dispute:' || id::text, 'dispute.filed',
           'Dispute filed on parcel ' || left(parcel_id::text, 8),
           (EXTRACT(EPOCH FROM filed_at) * 1000)::bigint, 'api'
      FROM disputes
    UNION ALL
    SELECT 'escrow:' || id::text, 'escrow.created',
           'Escrow created (' || status || ')',
           (EXTRACT(EPOCH FROM created_at) * 1000)::bigint, 'api'
      FROM escrows
    UNION ALL
    SELECT 'validator:' || registry_pubkey || '/' || validator_pubkey, 'validator.heartbeat',
           'Validator ' || left(registry_pubkey, 10) || ' heartbeat',
           last_active * 1000, 'api'
      FROM validator_activities
) t ORDER BY at DESC LIMIT $1
"#;

pub async fn list(
    Query(params): Query<ListParams>,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let limit = clamp_limit(params.limit);
    let rows: Vec<ActivityItem> = sqlx::query_as::<_, ActivityItem>(FEED_SQL)
        .bind(limit)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(serde_json::json!({
        "items": rows,
        "source": "offchain-api",
        "note": "real rows written by the API — on-chain Solana txs are not indexed yet",
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_defaults_and_clamps() {
        assert_eq!(clamp_limit(None), 50);
        assert_eq!(clamp_limit(Some(0)), 1);
        assert_eq!(clamp_limit(Some(-5)), 1);
        assert_eq!(clamp_limit(Some(10)), 10);
        assert_eq!(clamp_limit(Some(10_000)), 200);
    }

    #[test]
    fn feed_sql_covers_all_six_sources() {
        for table in [
            "FROM parcels",
            "FROM evidence_uploads",
            "FROM rights",
            "FROM disputes",
            "FROM escrows",
            "FROM validator_activities",
        ] {
            assert!(FEED_SQL.contains(table), "feed SQL missing {table}");
        }
        assert!(FEED_SQL.contains("ORDER BY at DESC LIMIT $1"));
        assert_eq!(FEED_SQL.matches("UNION ALL").count(), 5);
    }

    #[test]
    fn item_serialises_with_expected_keys() {
        let item = ActivityItem {
            id: "parcel:abc".into(),
            kind: "parcel.created".into(),
            summary: "Parcel abc registered".into(),
            at: 1_700_000_000_000,
            source: "api".into(),
        };
        let v = serde_json::to_value(&item).expect("serialize");
        assert_eq!(v["id"], "parcel:abc");
        assert_eq!(v["kind"], "parcel.created");
        assert_eq!(v["at"], 1_700_000_000_000_i64);
        assert_eq!(v["source"], "api");
    }
}
