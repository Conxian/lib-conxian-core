# Conxian Gateway API Reference v2

## 1. System Endpoints

### GET /api/v1/health
Returns the health status.
- **Response:** `{ "status": "healthy", "engine": "active" }`

### GET /api/v1/status
Returns general system information.
- **Response:** `{ "version": "0.2.0", "uptime_seconds": 1234, "status": "operational", "total_requests": 5678, "total_tvl_usd": 1320000000 }`

### GET /api/v1/risk-assessment
Returns a detailed risk assessment for all layers.

### GET /api/v1/metrics
Prometheus-compatible metrics endpoint.

## 2. Protocol-Specific Endpoints

### Core DAO
- **GET /api/v1/core-dao/stats**: Satoshi Plus specific metrics.

### Lorenzo
- **GET /api/v1/lorenzo/stats**: Staking and yield metrics.

### Hemi
- **GET /api/v1/hemi/status**: Sequencer and finality status.

### BOB (Build on Bitcoin)
- **GET /api/v1/bob/info**: TVL and bridge status.

### Merlin Chain
- **GET /api/v1/merlin/stats**: ZK proving and yield stats.

### Mezo
- **GET /api/v1/mezo/yield**: Economic security and yield info.

### Nubit
- **GET /api/v1/nubit/da**: Data availability throughput and nodes.

### Bison
- **GET /api/v1/bison/stats**: ZK Rollup uptime and settlement frequency.

### Zulu Network
- **GET /api/v1/zulu/info**: Layer type and bridge mode.

### Botanix
- **GET /api/v1/botanix/stats**: Spiderchain nodes and multisig threshold.

### Bitlayer
- **GET /api/v1/bitlayer/info**: BitVM challenge status and validator count.

### Alpen
- **GET /api/v1/alpen/stats**: ZK proof type and batch size.

### Taproot Assets
- **GET /api/v1/taproot-assets/stats**: Asset issuance and transfer stats.

### BitVM2
- **GET /api/v1/bitvm2/info**: ZK-Fraud Proof paradigm details.

## 3. Data Models

### ServiceStatus
```json
{
  "name": "stacks",
  "status": "active",
  "latency_ms": 65,
  "trust_model": "PoX",
  "risk_level": "Medium",
  "risk_assessment": { ... },
  "data_availability": "On-chain",
  "settlement": "Bitcoin",
  "bridge_security": "sBTC Bridge",
  "tvl_usd": 12500000.0,
  "metadata": { ... }
}
```
