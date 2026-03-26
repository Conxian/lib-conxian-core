# Conxian Gateway API Reference v2

## 1. System Endpoints

### GET /api/v1/health
Returns the health status.
- **Response:** `{ "status": "healthy", "engine": "active" }`

### GET /api/v1/status
Returns general system information with high-precision metrics.
- **Response:** `{ "version": "0.2.0", "uptime_seconds": 1234, "status": "operational", "total_requests": 5678, "total_tvl_usd": 1320000000.55 }`

### GET /api/v1/risk-assessment
Returns a detailed multi-factor risk assessment (DA, Settlement, Bridge, Exit Mechanism, Operators) for all layers.

### GET /api/v1/metrics
Prometheus-compatible metrics including service latency and granular risk scores.

### POST /api/v1/compliance/check
Check address compliance status.
- **Request:** `{ "address": "bc1q..." }`

### POST /api/v1/compliance/zkml-verify
Verify a Zero-Knowledge Machine Learning proof (CON-70).
- **Request:** `{ "proof": "zkml_..." }`
- **Response:** `{ "verified": true, "attestation_role": "Guardian", "compliance_standard": "CARF/BRS v1.5", ... }`

## 2. Protocol-Specific Endpoints

### Stacks
- **Real-time:** Includes `block_height` and `hiro_api_connected` in metadata.

### Liquid / Rootstock
- **Reserves:** Dynamic collateral ratio tracking and "Verified (On-chain)" status.

### BitVM2
- **Challenges:** Real-time `bitvm_challenge_status` monitoring integrated into risk assessment.

### Core DAO
- **GET /api/v1/core-dao/stats**: Satoshi Plus specific metrics and validator status.

### Lorenzo
- **GET /api/v1/lorenzo/stats**: Staking yield and reward token info.

### Hemi
- **GET /api/v1/hemi/status**: Bitcoin/Ethereum finality depth and sequencer status.

### BOB (Build on Bitcoin)
- **GET /api/v1/bob/info**: Multi-chain bridge status and optimistic exit period.

### Merlin Chain
- **GET /api/v1/merlin/stats**: ZK proving status and user activity.

### Mezo
- **GET /api/v1/mezo/yield**: tBTC economic security and APY.

### Nubit
- **GET /api/v1/nubit/da**: DA throughput (MBps) and active DA nodes.

### Bison
- **GET /api/v1/bison/stats**: ZK Rollup uptime and proof latency.

### Zulu Network
- **GET /api/v1/zulu/info**: Layer hierarchy and decentralized bridge mode.

### Botanix
- **GET /api/v1/botanix/stats**: Spiderchain node count and multisig thresholds.

### Bitlayer
- **GET /api/v1/bitlayer/info**: BitVM challenge state and block metrics.

### Alpen
- **GET /api/v1/alpen/stats**: ZK proof type (SNARK) and settlement frequency.

### Taproot Assets
- **GET /api/v1/taproot-assets/stats**: Issuance stats and Lightning integration.

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
