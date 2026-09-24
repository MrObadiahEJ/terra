use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// RFC-012 Phase 3 — verification tasks
//
// Goal: `Task → requirements → routing → assignment → verification → reward`.
// PDA seeds per RFC-012 §10. Full multi-factor routing is Phase 6; Phase 3
// ships explicit requester assignment + first-come claim. Reward lamports are
// recorded on the task; on-chain escrow/distribution is Phase 8.
// Migration map (§9): `Claim` remains the legacy subject; new tasks reference
// claims (or parcels/identities) via `subject`.
// ---------------------------------------------------------------------------

/// Max requirement rows per task.
pub const MAX_TASK_REQUIREMENTS: u8 = 8;
/// Max concurrent assignments per task.
pub const MAX_TASK_VALIDATORS: u8 = 8;
/// Minimum validators required to complete a task.
pub const MIN_TASK_VALIDATORS: u8 = 1;

/// Capability code meaning "any capability" (not in taxonomy).
pub const CAPABILITY_ANY: u8 = 255;

pub mod task_class {
    /// Requires physical presence at a location.
    pub const PHYSICAL: u8 = 0;
    /// Can be done remotely (phone, desktop, API).
    pub const REMOTE: u8 = 1;
    /// Documentary review (deeds, IDs, scans).
    pub const DOCUMENTARY: u8 = 2;
    /// Computational / data check.
    pub const COMPUTATIONAL: u8 = 3;
    /// Mixed physical + remote + documentary.
    pub const HYBRID: u8 = 4;
    pub const MAX: u8 = HYBRID;
}

pub mod task_status {
    /// Created; open for assignment / claims.
    pub const OPEN: u8 = 0;
    /// At least one validator assigned; not yet complete.
    pub const ASSIGNED: u8 = 1;
    /// Work underway (optional intermediate; set when first result lands short of quorum).
    pub const IN_PROGRESS: u8 = 2;
    /// Required number of matching results recorded.
    pub const COMPLETED: u8 = 3;
    /// Cancelled by requester before completion.
    pub const CANCELLED: u8 = 4;
    /// Deadline passed without completion (lazy or explicit).
    pub const EXPIRED: u8 = 5;
    pub const MAX: u8 = EXPIRED;
}

pub mod task_outcome {
    /// No result yet.
    pub const PENDING: u8 = 0;
    pub const PASS: u8 = 1;
    pub const FAIL: u8 = 2;
    pub const INCONCLUSIVE: u8 = 3;
    pub const MAX: u8 = INCONCLUSIVE;
}

pub mod assignment_status {
    pub const ASSIGNED: u8 = 0;
    pub const SUBMITTED: u8 = 1;
    pub const RELEASED: u8 = 2;
    pub const MAX: u8 = RELEASED;
}

// ---------------------------------------------------------------------------
// Accounts (PDA seeds per RFC-012 §10)
// ---------------------------------------------------------------------------

/// First-class work unit (requester, subject, class, reward, deadline).
///
/// PDA: `["task", task_id]`
#[account]
#[derive(InitSpace)]
pub struct VerificationTask {
    /// Client-provided unique id (e.g. SHA-256 of the task statement).
    pub task_id: [u8; 32],
    /// Who created and funds the task (pays rent; Phase 8 escrow source).
    pub requester: Pubkey,
    /// What the task is about — claim, parcel, identity, or other account.
    /// Not constrained to one program (migration map: claims become subjects).
    pub subject: Pubkey,
    /// One of `task_class`.
    pub task_class: u8,
    /// One of `task_status`.
    pub status: u8,
    /// Reward in lamports recorded for Phase 8 distribution (0 = unfunded).
    pub reward_lamports: u64,
    /// Unix ts after which the task cannot be claimed/assigned/submitted.
    pub deadline: i64,
    /// sha256 of off-chain task description / requirements narrative.
    pub description_hash: [u8; 32],
    /// Number of `TaskRequirement` rows (append-only via req_index).
    pub requirement_count: u8,
    /// Number of `TaskAssignment` rows created.
    pub assigned_count: u8,
    /// How many submitted results are needed to complete (1..=MAX_TASK_VALIDATORS).
    pub required_validators: u8,
    /// How many results have been submitted so far.
    pub result_count: u8,
    /// One of `task_outcome` (PENDING until first result).
    pub outcome: u8,
    /// Canonical hash of the aggregated result / report.
    pub result_hash: [u8; 32],
    pub created_at: i64,
    pub updated_at: i64,
    /// 0 until COMPLETED.
    pub completed_at: i64,
}

/// One row of eligibility / evidence constraints for a task.
///
/// PDA: `["task_requirement", task_id, req_index]`
#[account]
#[derive(InitSpace)]
pub struct TaskRequirement {
    pub task_id: [u8; 32],
    /// Append-only index (0, 1, … requirement_count-1).
    pub req_index: u8,
    /// Required capability (`capability_code`) or `CAPABILITY_ANY`.
    pub capability_code: u8,
    /// Minimum legacy `ValidatorReputation.reputation_score` (0 = none).
    pub min_reputation: u16,
    /// Minimum `ValidatorProfile.tier` (0 = any).
    pub min_tier: u8,
    /// ISO 3166-1 alpha-2 as two bytes; `[0,0]` = any jurisdiction.
    pub jurisdiction: [u8; 2],
    /// Max distance from center in meters; 0 = no geographic constraint.
    pub radius_m: u32,
    /// Geographic center (degrees * 1e7) when `radius_m > 0`.
    pub center_lat_e7: i32,
    pub center_lon_e7: i32,
    /// Preferred independence (relationship-graph) threshold in bps (0 = none).
    pub independence_bps: u16,
    /// Minimum confidence the result should target (0–10000 bps).
    pub confidence_target_bps: u16,
    pub created_at: i64,
}

/// Selected validator for a task (requester assignment or self-claim).
///
/// PDA: `["task_assignment", task_id, validator]`
#[account]
#[derive(InitSpace)]
pub struct TaskAssignment {
    pub task_id: [u8; 32],
    /// Assigned validator wallet.
    pub validator: Pubkey,
    /// Requester wallet that created the task (audit trail).
    pub requester: Pubkey,
    /// Who performed the assignment: requester = assign, validator = claim.
    pub assigned_by: Pubkey,
    /// One of `assignment_status`.
    pub status: u8,
    pub assigned_at: i64,
    /// 0 until `submit_task_result`.
    pub submitted_at: i64,
    pub updated_at: i64,
}

// ---------------------------------------------------------------------------
// Pure helpers (unit-testable)
// ---------------------------------------------------------------------------

pub fn is_valid_task_class(class: u8) -> bool {
    class <= task_class::MAX
}

pub fn is_valid_task_status(status: u8) -> bool {
    status <= task_status::MAX
}

pub fn is_valid_task_outcome(outcome: u8) -> bool {
    outcome <= task_outcome::MAX
}

pub fn is_valid_assignment_status(status: u8) -> bool {
    status <= assignment_status::MAX
}

/// Capability requirement is valid taxonomy code or ANY.
pub fn is_valid_requirement_capability(code: u8) -> bool {
    code == CAPABILITY_ANY || crate::validator_profile::is_valid_capability_code(code)
}

/// Terminal statuses accept no further assignment / claims / results.
pub fn task_is_terminal(status: u8) -> bool {
    matches!(
        status,
        task_status::COMPLETED | task_status::CANCELLED | task_status::EXPIRED
    )
}

/// OPEN accepts assignment and claims.
pub fn task_is_open(status: u8) -> bool {
    status == task_status::OPEN
}

/// Past deadline (strict: now >= deadline).
pub fn task_is_past_deadline(now: i64, deadline: i64) -> bool {
    now >= deadline
}

/// Geographic bounds same as presence (|lat|<=90e7, |lon|<=180e7).
pub fn is_valid_geo_center(lat_e7: i32, lon_e7: i32) -> bool {
    lat_e7.abs() <= 900_000_000 && lon_e7.abs() <= 1_800_000_000
}

/// Confidence target in [0, 10000] bps.
pub fn is_valid_confidence_bps(bps: u16) -> bool {
    bps <= 10_000
}

/// `required_validators` in 1..=MAX_TASK_VALIDATORS.
pub fn is_valid_required_validators(n: u8) -> bool {
    (MIN_TASK_VALIDATORS..=MAX_TASK_VALIDATORS).contains(&n)
}

/// Assignment allowed only when task is OPEN or ASSIGNED/IN_PROGRESS and not past deadline.
pub fn can_assign(now: i64, status: u8, deadline: i64) -> bool {
    !task_is_terminal(status)
        && !task_is_past_deadline(now, deadline)
        && (task_is_open(status)
            || status == task_status::ASSIGNED
            || status == task_status::IN_PROGRESS)
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Create a verification task. Permissionless; requester signs and pays rent.
#[allow(clippy::too_many_arguments)]
pub fn create_verification_task(
    ctx: Context<crate::CreateVerificationTask>,
    task_id: [u8; 32],
    subject: Pubkey,
    task_class: u8,
    reward_lamports: u64,
    deadline: i64,
    description_hash: [u8; 32],
    required_validators: u8,
) -> Result<()> {
    require!(
        is_valid_task_class(task_class),
        TerraError::InvalidTaskClass
    );
    require!(
        is_valid_required_validators(required_validators),
        TerraError::InvalidTaskRequirement
    );
    require!(task_id != [0u8; 32], TerraError::InvalidTaskRequirement);
    let now = Clock::get()?.unix_timestamp;
    require!(deadline > now, TerraError::TaskDeadlinePassed);

    let t = &mut ctx.accounts.task;
    t.task_id = task_id;
    t.requester = ctx.accounts.requester.key();
    t.subject = subject;
    t.task_class = task_class;
    t.status = task_status::OPEN;
    t.reward_lamports = reward_lamports;
    t.deadline = deadline;
    t.description_hash = description_hash;
    t.requirement_count = 0;
    t.assigned_count = 0;
    t.required_validators = required_validators;
    t.result_count = 0;
    t.outcome = task_outcome::PENDING;
    t.result_hash = [0u8; 32];
    t.created_at = now;
    t.updated_at = now;
    t.completed_at = 0;

    emit!(crate::VerificationTaskCreated {
        task: t.key(),
        task_id,
        requester: t.requester,
        subject,
        task_class,
        deadline,
        required_validators,
    });
    Ok(())
}

/// Append one requirement row. Requester only; task must be OPEN;
/// `req_index` must equal current `requirement_count` (append-only).
#[allow(clippy::too_many_arguments)]
pub fn add_task_requirement(
    ctx: Context<crate::AddTaskRequirement>,
    req_index: u8,
    capability_code: u8,
    min_reputation: u16,
    min_tier: u8,
    jurisdiction: [u8; 2],
    radius_m: u32,
    center_lat_e7: i32,
    center_lon_e7: i32,
    independence_bps: u16,
    confidence_target_bps: u16,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let t = &ctx.accounts.task;
    require!(
        t.requester == ctx.accounts.requester.key(),
        TerraError::NotTaskRequester
    );
    require!(task_is_open(t.status), TerraError::TaskNotOpen);
    require!(
        req_index == t.requirement_count,
        TerraError::RequirementIndexMismatch
    );
    require!(
        t.requirement_count < MAX_TASK_REQUIREMENTS,
        TerraError::TaskRequirementsFull
    );
    require!(
        is_valid_requirement_capability(capability_code),
        TerraError::InvalidCapabilityCode
    );
    require!(
        min_tier <= crate::validator_profile::profile_tier::MAX,
        TerraError::InvalidProfileTier
    );
    require!(
        is_valid_confidence_bps(confidence_target_bps),
        TerraError::InvalidConfidence
    );
    if radius_m > 0 {
        require!(
            is_valid_geo_center(center_lat_e7, center_lon_e7),
            TerraError::InvalidPresenceFix
        );
    }

    let r = &mut ctx.accounts.requirement;
    r.task_id = t.task_id;
    r.req_index = req_index;
    r.capability_code = capability_code;
    r.min_reputation = min_reputation;
    r.min_tier = min_tier;
    r.jurisdiction = jurisdiction;
    r.radius_m = radius_m;
    r.center_lat_e7 = center_lat_e7;
    r.center_lon_e7 = center_lon_e7;
    r.independence_bps = independence_bps;
    r.confidence_target_bps = confidence_target_bps;
    r.created_at = now;

    let task = &mut ctx.accounts.task;
    task.requirement_count = task.requirement_count.saturating_add(1);
    task.updated_at = now;

    emit!(crate::TaskRequirementAdded {
        task: task.key(),
        task_id: task.task_id,
        req_index,
        capability_code,
        min_reputation,
        min_tier,
    });
    Ok(())
}

/// Requester explicitly assigns a validator (Phase 3 routing step).
/// Task must be OPEN/ASSIGNED/IN_PROGRESS, not past deadline, not full.
pub fn assign_task_validator(
    ctx: Context<crate::AssignTaskValidator>,
    _task_id: [u8; 32],
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let t = &ctx.accounts.task;
    require!(
        t.requester == ctx.accounts.requester.key(),
        TerraError::NotTaskRequester
    );
    require!(
        can_assign(now, t.status, t.deadline),
        if task_is_past_deadline(now, t.deadline) {
            TerraError::TaskDeadlinePassed
        } else if task_is_terminal(t.status) {
            TerraError::TaskAlreadyFinalized
        } else {
            TerraError::TaskNotOpen
        }
    );
    let validator = ctx.accounts.validator.key();
    require!(validator != t.requester, TerraError::SelfTaskAssignment);
    require!(
        t.assigned_count < t.required_validators,
        TerraError::TaskAlreadyAssigned
    );

    let task = &mut ctx.accounts.task;
    task.assigned_count = task.assigned_count.saturating_add(1);
    if task.assigned_count == 1 {
        task.status = task_status::ASSIGNED;
    }
    task.updated_at = now;

    let a = &mut ctx.accounts.assignment;
    a.task_id = task.task_id;
    a.validator = validator;
    a.requester = task.requester;
    a.assigned_by = task.requester;
    a.status = assignment_status::ASSIGNED;
    a.assigned_at = now;
    a.submitted_at = 0;
    a.updated_at = now;

    emit!(crate::TaskAssigned {
        task: task.key(),
        task_id: task.task_id,
        validator,
        assigned_by: task.requester,
    });
    Ok(())
}

/// Validator first-come claims an open slot (simple Phase 3 routing).
pub fn claim_task(ctx: Context<crate::ClaimTask>, _task_id: [u8; 32]) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let t = &ctx.accounts.task;
    require!(
        can_assign(now, t.status, t.deadline),
        if task_is_past_deadline(now, t.deadline) {
            TerraError::TaskDeadlinePassed
        } else if task_is_terminal(t.status) {
            TerraError::TaskAlreadyFinalized
        } else {
            TerraError::TaskNotOpen
        }
    );
    let validator = ctx.accounts.validator.key();
    require!(validator != t.requester, TerraError::SelfTaskAssignment);
    require!(
        t.assigned_count < t.required_validators,
        TerraError::TaskAlreadyAssigned
    );

    let task = &mut ctx.accounts.task;
    task.assigned_count = task.assigned_count.saturating_add(1);
    if task.assigned_count == 1 {
        task.status = task_status::ASSIGNED;
    }
    task.updated_at = now;

    let a = &mut ctx.accounts.assignment;
    a.task_id = task.task_id;
    a.validator = validator;
    a.requester = task.requester;
    a.assigned_by = validator;
    a.status = assignment_status::ASSIGNED;
    a.assigned_at = now;
    a.submitted_at = 0;
    a.updated_at = now;

    emit!(crate::TaskAssigned {
        task: task.key(),
        task_id: task.task_id,
        validator,
        assigned_by: validator,
    });
    Ok(())
}

/// Assigned validator submits a result. When `result_count` reaches
/// `required_validators`, the task completes with this outcome (first
/// non-pending outcome wins; later mismatches are ignored for Phase 3 —
/// full quorum aggregation is Phase 6/7).
pub fn submit_task_result(
    ctx: Context<crate::SubmitTaskResult>,
    _task_id: [u8; 32],
    outcome: u8,
    result_hash: [u8; 32],
) -> Result<()> {
    require!(
        matches!(
            outcome,
            task_outcome::PASS | task_outcome::FAIL | task_outcome::INCONCLUSIVE
        ),
        TerraError::InvalidTaskOutcome
    );
    let now = Clock::get()?.unix_timestamp;
    let t = &ctx.accounts.task;
    require!(
        !task_is_terminal(t.status),
        TerraError::TaskAlreadyFinalized
    );
    require!(
        !task_is_past_deadline(now, t.deadline),
        TerraError::TaskDeadlinePassed
    );
    require!(
        ctx.accounts.validator.key() == ctx.accounts.assignment.validator,
        TerraError::NotTaskAssignee
    );
    require!(
        ctx.accounts.assignment.status == assignment_status::ASSIGNED,
        TerraError::TaskAlreadyFinalized
    );
    require!(t.assigned_count > 0, TerraError::NotTaskAssignee);

    let now = Clock::get()?.unix_timestamp;
    let assignment = &mut ctx.accounts.assignment;
    assignment.status = assignment_status::SUBMITTED;
    assignment.submitted_at = now;
    assignment.updated_at = now;

    let task = &mut ctx.accounts.task;
    task.result_count = task.result_count.saturating_add(1);
    if task.outcome == task_outcome::PENDING {
        task.outcome = outcome;
        task.result_hash = result_hash;
    }
    if task.result_count >= task.required_validators {
        task.status = task_status::COMPLETED;
        task.completed_at = now;
    } else {
        task.status = task_status::IN_PROGRESS;
    }
    task.updated_at = now;

    emit!(crate::TaskResultSubmitted {
        task: task.key(),
        task_id: task.task_id,
        validator: assignment.validator,
        outcome,
        result_count: task.result_count,
        completed: task.status == task_status::COMPLETED,
    });
    Ok(())
}

/// Requester cancels an incomplete task (OPEN / ASSIGNED / IN_PROGRESS).
pub fn cancel_task(ctx: Context<crate::CancelTask>, _task_id: [u8; 32]) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let t = &mut ctx.accounts.task;
    require!(
        t.requester == ctx.accounts.requester.key(),
        TerraError::NotTaskRequester
    );
    require!(
        !task_is_terminal(t.status),
        TerraError::TaskAlreadyFinalized
    );
    t.status = task_status::CANCELLED;
    t.updated_at = now;

    emit!(crate::TaskCancelled {
        task: t.key(),
        task_id: t.task_id,
        requester: t.requester,
        cancelled_at: now,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_class_constants_are_contiguous() {
        assert_eq!(task_class::PHYSICAL, 0);
        assert_eq!(task_class::REMOTE, 1);
        assert_eq!(task_class::DOCUMENTARY, 2);
        assert_eq!(task_class::COMPUTATIONAL, 3);
        assert_eq!(task_class::HYBRID, 4);
        for c in 0..=task_class::MAX {
            assert!(is_valid_task_class(c));
        }
        assert!(!is_valid_task_class(5));
    }

    #[test]
    fn task_status_constants_are_contiguous() {
        assert_eq!(task_status::OPEN, 0);
        assert_eq!(task_status::ASSIGNED, 1);
        assert_eq!(task_status::IN_PROGRESS, 2);
        assert_eq!(task_status::COMPLETED, 3);
        assert_eq!(task_status::CANCELLED, 4);
        assert_eq!(task_status::EXPIRED, 5);
        for s in 0..=task_status::MAX {
            assert!(is_valid_task_status(s));
        }
        assert!(!is_valid_task_status(6));
        assert!(task_is_open(task_status::OPEN));
        assert!(!task_is_open(task_status::ASSIGNED));
        assert!(task_is_terminal(task_status::COMPLETED));
        assert!(task_is_terminal(task_status::CANCELLED));
        assert!(task_is_terminal(task_status::EXPIRED));
        assert!(!task_is_terminal(task_status::OPEN));
        assert!(!task_is_terminal(task_status::ASSIGNED));
        assert!(!task_is_terminal(task_status::IN_PROGRESS));
    }

    #[test]
    fn task_outcome_constants() {
        assert_eq!(task_outcome::PENDING, 0);
        assert_eq!(task_outcome::PASS, 1);
        assert_eq!(task_outcome::FAIL, 2);
        assert_eq!(task_outcome::INCONCLUSIVE, 3);
        for o in 0..=task_outcome::MAX {
            assert!(is_valid_task_outcome(o));
        }
        assert!(!is_valid_task_outcome(4));
        assert!(matches!(
            task_outcome::PASS | task_outcome::FAIL | task_outcome::INCONCLUSIVE,
            1..=3
        ));
    }

    #[test]
    fn assignment_status_and_capability_any() {
        assert_eq!(assignment_status::ASSIGNED, 0);
        assert_eq!(assignment_status::SUBMITTED, 1);
        assert_eq!(assignment_status::RELEASED, 2);
        for s in 0..=assignment_status::MAX {
            assert!(is_valid_assignment_status(s));
        }
        assert!(is_valid_requirement_capability(CAPABILITY_ANY));
        assert!(is_valid_requirement_capability(
            crate::validator_profile::capability_code::GNSS
        ));
        assert!(!is_valid_requirement_capability(
            crate::validator_profile::capability_code::MAX + 1
        ));
    }

    #[test]
    fn deadline_and_assign_gates() {
        assert!(task_is_past_deadline(100, 100));
        assert!(task_is_past_deadline(101, 100));
        assert!(!task_is_past_deadline(99, 100));

        // OPEN before deadline: assignable
        assert!(can_assign(50, task_status::OPEN, 100));
        // OPEN past deadline: not assignable
        assert!(!can_assign(150, task_status::OPEN, 100));
        // COMPLETED: not assignable
        assert!(!can_assign(50, task_status::COMPLETED, 100));
        // ASSIGNED before deadline: still assignable (more slots)
        assert!(can_assign(50, task_status::ASSIGNED, 100));
        // CANCELLED: not assignable
        assert!(!can_assign(50, task_status::CANCELLED, 100));
    }

    #[test]
    fn required_validators_and_geo_bounds() {
        assert!(is_valid_required_validators(1));
        assert!(is_valid_required_validators(MAX_TASK_VALIDATORS));
        assert!(!is_valid_required_validators(0));
        assert!(!is_valid_required_validators(MAX_TASK_VALIDATORS + 1));

        assert!(is_valid_geo_center(0, 0));
        assert!(is_valid_geo_center(387_500_000, 121_500_000));
        assert!(!is_valid_geo_center(900_000_001, 0));
        assert!(!is_valid_geo_center(0, 1_800_000_001));

        assert!(is_valid_confidence_bps(0));
        assert!(is_valid_confidence_bps(10_000));
        assert!(!is_valid_confidence_bps(10_001));
    }
}
