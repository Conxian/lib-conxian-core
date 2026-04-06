# Conxian Gateway API Reference v2

## 1. System Endpoints

### GET /api/v1/health
Returns the health status.
- **Response:** `{ "status": "healthy", "engine": "active" }`

### GET /api/v1/status
Returns general system information with high-precision metrics.
- **Response:** `{ "version": "0.2.0", "uptime_seconds": 1234, "status": "operational", "total_requests": 5678, "total_tvl_usd": 1320000000.55 }`

### GET /api/v1/financials
Returns real-time financial intelligence metrics (MRR, ARR, Churn).
- **Response:** `{ "mrr_usd": 125000.0, "arr_usd": 1500000.0, "churn_rate_pct": 2.5, "protocol_fees_collected_usd": 85000.0, "last_updated": "..." }`

### GET /api/v1/risk-assessment
Returns a detailed multi-factor risk assessment (DA, Settlement, Bridge, Exit Mechanism, Operators) for all layers.

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

### POST /api/v1/erp/sync
Synchronizes institutional ERP data (SAP/Oracle) with the Conxian ledger (CON-63).
- **Request:** `{ "system": "SAP" }`

### GET /api/v1/spec/cjcs
Returns the CJCS v2.0 JSON-LD machine-readable definition (CON-73).

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
```json
{
  "name": "stacks",
  "status": "active",
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

## 4. Data Models (Continued)

### StateProposal
```json
{
  "proposal_id": "prop-iso20022-trigger-1712412345",
  "trigger_id": "iso20022-trigger-1712412345",
  "proposed_state": "SovereignStateUpdate",
  "timelock_end_block": 841144,
  "status": "Pending",
  "tee_attestation": "VerifiedByStrongBox-v1.0"
}
```
