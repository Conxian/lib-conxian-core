# Conxian Gateway API Reference v0.2.5

## 1. System Endpoints

### Production Branch Policy (CON-407)
All production-facing endpoints (/api/v1/settlement/*, /api/v1/erp/*, /api/v1/state/*) enforce mainnet-only behavior. In production environments (CONXIAN_NETWORK=mainnet), testnet bypasses are strictly prohibited. Non-production environments MUST provide an explicit `testnet` flag in the JSON payload to acknowledge validation state.

### Administrative Authentication Policy
High-privilege endpoints (Proposal Approval, Execution, ERP Sync, State Commit, Lightning Payments, and the MCP surface) require administrative authentication via the `X-Gateway-Admin-Key` header. The expected value must be configured in the `GATEWAY_ADMIN_API_KEY` environment variable. If the environment variable is not set, these endpoints will return `503 Service Unavailable`.


### GET /api/v1/health
Returns the health status.
- **Response:** `{ "status": "healthy", "engine": "active" }`

### GET /api/v1/status
Returns general system information with high-precision metrics.
- **Response:** `{ "version": "0.2.5", "uptime_seconds": 1234, "status": "operational", "total_requests": 5678, "total_tvl_usd": 1320000000.55 }`

### GET /api/v1/financials
Returns real-time financial intelligence metrics (MRR, ARR, Churn).
- **Response:** `{ "mrr_usd": 125000.0, "arr_usd": 1500000.0, "churn_rate_pct": 2.5, "protocol_fees_collected_usd": 85000.0, "last_updated": "..." }`

### GET /api/v1/risk-assessment
Returns a detailed multi-factor risk assessment (DA, Settlement, Bridge, Exit Mechanism, Operators) for all layers.

### GET /api/v1/layers
Returns an array of `ServiceStatus` objects for all registered rails.

### GET /api/v1/metrics
Prometheus-compatible metrics including service latency and granular risk scores.

### POST /api/v1/compliance/check
Check address compliance status.
- **Request:** `{ "address": "bc1q..." }`

### POST /api/v1/compliance/zkml-verify
Verify a Zero-Knowledge Machine Learning proof (CON-70). Aligned with CARF/BRS v1.5.
- **Request:** `{ "proof": "zkml_..." }`
- **Response:** `{ "verified": true, "attestation_role": "Guardian", "compliance_standard": "CARF/BRS v1.5", ... }`

### GET /api/v1/identity/{query}
Resolves identity via ENS, BNS, and World ID (CON-66).
- **Response:** `{ "address": "...", "ens_name": "...", "bns_name": "...", "world_id_verified": true }`


### POST /api/v1/lightning/pay
Pays a Lightning Network invoice. Mainnet-only.
- **Request:** `{ "invoice": "lnbc...", "testnet": true }`

### POST /api/v1/erp/sync
Synchronizes institutional ERP data (SAP/Oracle) with the Conxian ledger (CON-63). Mainnet-only.
- **Request:** `{ "system": "SAP", "testnet": true }`

### GET /api/v1/spec/cjcs
Returns the CJCS v0.2.5 JSON-LD machine-readable definition (CON-73).

### GET /api/v1/finance/bond/{id}
Retrieves Bitcoin DLC Bond lifecycle information (CON-62, CON-72).

### POST /api/v1/state/commit
Commits state shards to Tableland for decentralized persistence (CON-69).

## 2. Protocol-Specific Endpoints

### Stacks
- **Real-time:** Includes `block_height` and `hiro_api_connected` in metadata.

### Liquid / Rootstock
- **Reserves:** Dynamic collateral ratio tracking and "Verified (On-chain)" status.

### BitVM2
- **Challenges:** Real-time `bitvm_challenge_status` monitoring integrated into risk assessment (CON-75).

### Other Supported Layers
- **Core DAO**, **Lorenzo**, **Hemi**, **BOB**, **Merlin Chain**, **Mezo**, **Nubit**, **Bison**, **Zulu Network**, **Botanix**, **Bitlayer**, **Alpen**, **Taproot Assets**.

## 3. Data Models

### ServiceStatus
`ServiceStatus` now includes typed `rail_metadata` for per-rail trust/finality/custody/compliance/capability context.

```json
{
  "name": "stacks",
  "status": "active",
  "last_checked": "2026-05-27T22:00:00Z",
  "latency_ms": 65,
  "trust_model": "PoX",
  "risk_level": "Medium",
  "risk_assessment": {
    "overall_level": "Medium",
    "da_score": 90,
    "settlement_score": 85,
    "bridge_score": 80,
    "exit_mechanism_score": 85,
    "operators_score": 80,
    "decentralization_score": 75
  },
  "data_availability": "On-chain",
  "settlement": "Bitcoin",
  "bridge_security": "sBTC Bridge",
  "tvl_usd": 12500000.0,
  "version": "1.2.0",
  "metadata": {
    "block_height": "841234",
    "hiro_api_connected": "true"
  },
  "rail_metadata": {
    "rail_family": "anchored_l2",
    "trust_assumptions": {
      "security_anchor": "PoX consensus anchored to Bitcoin checkpointing",
      "operator_dependency": "Validator set and bridge operators maintain canonical state",
      "liveness_assumption": "Anchor publication and validator participation remain online"
    },
    "finality_semantics": {
      "confirmation_model": "Economic finality on L2, hardened by Bitcoin anchoring",
      "settlement_layer": "Bitcoin",
      "typical_finality_window": "~10-60 minutes depending on anchor cadence"
    },
    "custody_model": {
      "asset_control_model": "Hybrid self-custody with bridge-locked representations",
      "signer_architecture": "sBTC Bridge",
      "redemption_path": "Bridge redemption back to native Bitcoin"
    },
    "compliance_constraints": {
      "baseline_controls": [
        "kyc_for_managed_entrypoints",
        "sanctions_screening"
      ],
      "jurisdictional_scope": "Gateway policy controls by operating jurisdiction",
      "monitoring_requirements": [
        "bridge_health_monitoring",
        "anchor_finality_checks"
      ]
    },
    "operational_capabilities": {
      "supported_flows": [
        "btc_bridging",
        "smart_contract_execution",
        "token_transfers"
      ],
      "integration_modes": [
        "gateway_service_status",
        "bridge_api"
      ],
      "resilience_features": [
        "checkpoint_reconciliation",
        "bridge_failover_runbooks"
      ]
    }
  }
}
```

### POST /api/v1/settlement/iso20022
Ingress point for ISO 20022 external settlement messages. Verified in TEE.
- **Request:** `{ "msg_id": "...", "amount": 1000, ... }`
- **Response:** StateProposal object.

### POST /api/v1/settlement/papss
Ingress point for PAPSS external settlement messages. Verified in TEE.
- **Request:** `{ "tx_id": "...", "currency": "USD", ... }`
- **Response:** StateProposal object.

### POST /api/v1/settlement/brics
Ingress point for BRICS external settlement messages. Verified in TEE.
- **Request:** `{ "payload": "..." }`
- **Response:** StateProposal object.

### GET /api/v1/settlement/proposals
Lists all pending state proposals generated from external triggers.
- **Response:** Array of StateProposal objects.

### POST /api/v1/settlement/proposals/{id}/approve
Approves a pending state proposal. Transition status to "Approved". Requires `X-Gateway-Admin-Key` authentication.

### POST /api/v1/settlement/proposals/{id}/execute
Executes an approved state proposal. Transition status to "Executed". Requires `X-Gateway-Admin-Key` authentication.

## 4. Data Models (Continued)

### StateProposal
```json
{
  "proposal_id": "prop-iso20022-trigger-1712412345",
  "trigger_id": "iso20022-trigger-1712412345",
  "proposed_state": "MainnetSovereignStateUpdate",
  "timelock_end_block": 841144,
  "status": "Pending",
  "tee_attestation": "VerifiedByStrongBox-Mainnet-v1.0",
  "yield_routing": "5/5/90",
  "capital_status": "TransitBond"
}
```

### GET /api/v1/sab/wallets
Returns the canonical wallet inventory for BOS and related system operations (CON-423).
- **Response:** Array of SabWallet objects.

## 5. Data Models (Continued)

### SabWallet
```json
{
  "address": "SPSZXAKV7DWTDZN2601WR31BM51BD3YTQWE97VRM",
  "role": "Execution",
  "owner": "Operator",
  "status": "Active",
  "quorum": "1-of-1",
  "spending_limit_usd": 10000.0
}
```

## 6. Agentic & Autonomous Surface (MCP)

The Gateway exposes a read-first Model Context Protocol (MCP) layer to enable programmatic trust for autonomous agents.

### POST /api/v1/mcp
Unified entry point for MCP interactions (Tools, Resources, Prompts). Requires `X-Gateway-Admin-Key` authentication.

#### MCP Tools
- **get_system_telemetry**: Retrieve real-time TVL and node status.
- **get_protocol_proof**: Audit specific protocol proofs (BitVM, Citrea).
- **get_yield_metrics**: Analyze non-dilutive financial health and yield streams.
- **list_industrial_intents**: Discover tool schemas for FSOC validation or settlement triggers.
- **draft_financial_intent**: Construct complex financial intents for human signing (Subject to 144-block timelock).
- **list_state_proposals**: List all current state proposals, including their status and TEE attestations.
- **approve_state_proposal**: Approve a pending state proposal (Human-in-the-loop validation).
- **execute_state_proposal**: Execute an approved state proposal after timelock expiration.

#### Execution Flow
1. **Agent Drafting**: AI Agent construct an intent using `draft_financial_intent`.
2. **Proposal Creation**: Gateway emits a `StateProposal` with `Pending` status.
3. **Governance Gate**: Mandatory 144-block timelock is applied.
4. **Human Handshake**: Conxius Wallet visualizes the state change for hardware approval via `approve_state_proposal`.
5. **Final Execution**: Transaction is executed via `execute_state_proposal` (internally invoking `lib-conclave-sdk` logic when integrated).

## 7. Partner Intake Endpoints

### POST /api/v1/intake/partner
Creates a new partner lead. Requires `X-Partner-Intake-Key` and `Idempotency-Key` headers.
- **Request:**
  ```json
  {
    "partner_name": "Acme Corp",
    "contact_name": "John Doe",
    "contact_email": "john@acme.com",
    "company_name": "Acme Services",
    "notes": "Interested in sBTC integration"
  }
  ```
- **Response:** PartnerLead object.

### GET /api/v1/intake/partner/{id}
Retrieves a specific partner lead by ID.
- **Response:** PartnerLead object.

### GET /api/v1/intake/partner
Lists partner leads with optional filtering.
- **Query Params:** `status`, `owner`

### POST /api/v1/intake/partner/{id}/status
Updates the status of a partner lead.
- **Request:**
  ```json
  {
    "status": "assigned",
    "owner": "admin@conxian-labs.com",
    "event_note": "Assigned to primary review"
  }
  ```
