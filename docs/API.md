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
- **Response:**
  ```json
  {
    "stacks": {
      "overall_level": "Medium",
      "da_score": 90,
      "settlement_score": 85,
      "bridge_score": 55,
      "exit_mechanism_score": 85,
      "operators_score": 80,
      "decentralization_score": 75
    }
  }
  ```


### GET /api/v1/core-dao/stats
Returns Satoshi Plus specific metrics for Core DAO.
- **Response:**
  ```json
  {
    "hashrate_contribution_pct": 15.4,
    "dual_token_staking": "enabled",
    "active_validators": 21,
    "total_staked_btc": 2500.0,
    "satoshi_plus_status": "Active"
  }
  ```

### GET /api/v1/layers
Returns a consolidated view of all supported layers.

## 2. Service Endpoints
- **GET /api/v1/bitvm2**
- **GET /api/v1/core-dao**
- **GET /api/v1/prices**
- **GET /api/v1/reserves**

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
