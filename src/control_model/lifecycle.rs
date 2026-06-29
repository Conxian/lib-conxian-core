use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WalletAuthorityClass {
    WalletOwner,
    Delegate,
    Guardian,
    ServiceOperator,
    Automation,
    Auditor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalletAuthority {
    pub authority_id: String,
    pub wallet_id: String,
    pub class: WalletAuthorityClass,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProtectedActionLifecycleState {
    Draft,
    PendingAuthorization,
    Timelocked,
    ReadyForExecution,
    Executed,
    Rejected,
    Cancelled,
    Expired,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TriggerLifecycleState {
    Received,
    Validated,
    MaterializedAsPendingAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PendingActionLifecycleState {
    AwaitingQuorum,
    QuorumSatisfied,
    Timelocked,
    Ready,
    Executing,
    Executed,
    Rejected,
    Cancelled,
    Expired,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimelockInvariant {
    pub created_at_block: u64,
    pub timelock_blocks: u32,
    pub not_before_block: u64,
}

impl TimelockInvariant {
    pub fn new(created_at_block: u64, timelock_blocks: u32) -> Self {
        Self {
            created_at_block,
            timelock_blocks,
            not_before_block: created_at_block.saturating_add(timelock_blocks as u64),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuorumInvariant {
    pub approvals_required: u16,
    pub eligible_approvers: u16,
}

impl QuorumInvariant {
    pub fn new(approvals_required: u16, eligible_approvers: u16) -> Self {
        Self {
            approvals_required,
            eligible_approvers,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtectedActionInvariantSet {
    pub timelock: TimelockInvariant,
    pub quorum: QuorumInvariant,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BitcoinTxLifecycleState {
    Draft,
    Signed,
    BroadcastPending,
    InMempool,
    PendingConfirmations,
    Confirmed,
    Finalized,
    Reorged,
    DeadLetter,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BitcoinTxLifecycleEvent {
    Sign,
    QueueBroadcast,
    MempoolObserved,
    ConfirmationsObserved,
    Finalize,
    ReorgDetected,
    MarkDeadLetter,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BitcoinTxLifecycleExecutionMode {
    Disabled,
    Shadow,
    Active,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BitcoinTxLifecycleRolloutMode {
    Shadow,
    Limited,
    Full,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BitcoinFeeBumpAction {
    None,
    Rbf,
    Cpfp,
    Escalate,
}

pub fn is_valid_protected_action_transition(
    from: &ProtectedActionLifecycleState,
    to: &ProtectedActionLifecycleState,
) -> bool {
    matches!(
        (from, to),
        (
            ProtectedActionLifecycleState::Draft,
            ProtectedActionLifecycleState::PendingAuthorization
        ) | (
            ProtectedActionLifecycleState::Draft,
            ProtectedActionLifecycleState::Cancelled
        ) | (
            ProtectedActionLifecycleState::PendingAuthorization,
            ProtectedActionLifecycleState::Timelocked
        ) | (
            ProtectedActionLifecycleState::PendingAuthorization,
            ProtectedActionLifecycleState::Rejected
        ) | (
            ProtectedActionLifecycleState::PendingAuthorization,
            ProtectedActionLifecycleState::Cancelled
        ) | (
            ProtectedActionLifecycleState::Timelocked,
            ProtectedActionLifecycleState::ReadyForExecution
        ) | (
            ProtectedActionLifecycleState::Timelocked,
            ProtectedActionLifecycleState::Expired
        ) | (
            ProtectedActionLifecycleState::Timelocked,
            ProtectedActionLifecycleState::Cancelled
        ) | (
            ProtectedActionLifecycleState::ReadyForExecution,
            ProtectedActionLifecycleState::Executed
        ) | (
            ProtectedActionLifecycleState::ReadyForExecution,
            ProtectedActionLifecycleState::Failed
        )
    )
}

pub fn validate_protected_action_transition(
    from: &ProtectedActionLifecycleState,
    to: &ProtectedActionLifecycleState,
) -> Result<(), String> {
    if is_valid_protected_action_transition(from, to) {
        return Ok(());
    }

    Err(format!(
        "Invalid protected action transition: {:?} -> {:?}",
        from, to
    ))
}
