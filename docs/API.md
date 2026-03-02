# Conxian Gateway API Reference

The Conxian Gateway API is accessible under the `/api/v1` prefix.

## 1. System Endpoints

### GET /api/v1/health
Returns the health status of the gateway and its engine.
- **Response:** `{ "status": "healthy", "engine": "active" }`

### GET /api/v1/status
Returns general system information, uptime, and request counts.
- **Response:**
  ```json
  {
    "version": "0.1.0",
    "uptime_seconds": 1234,
    "status": "operational",
    "total_requests": 5678,
    "active_nodes": 8
  }
  ```

### GET /api/v1/metrics
Returns Prometheus-compatible metrics.
- **Response:** Text/plain metrics.

### GET /api/v1/compliance
Returns the current network compliance status and active rules.
- **Response:** `{ "status": "compliant", "last_audit": "...", "rules_active": ["KYC", "AML", "NetworkIntegrity"], "risk_score": 15 }`

### POST /api/v1/compliance/check
Performs a simulated AML/KYC check on a Bitcoin address.
- **Request Body:** `{ "address": "bc1q..." }`
- **Response:** `{ "address": "bc1q...", "compliant": true, "risk_score": 10, "timestamp": "..." }`

### GET /api/v1/affiliates
Returns information about active affiliate partners.
- **Response:** A JSON object where keys are partner IDs and values are `AffiliateInfo` objects.

### GET /api/v1/marketing
Returns information about active marketing channels and campaigns.
- **Response:** An array of `MarketingInfo` objects.

## 2. Service Endpoints

### GET /api/v1/layers
Returns a consolidated view of all supported Bitcoin layers and sovereign services.
- **Response:** A JSON object where keys are service names and values are `ServiceStatus` objects.

### GET /api/v1/reserves
Returns real-time audit data for pegged assets and bridge collateral ratios.
- **Response:** An array of `ReserveAsset` objects.

### Individual Service Endpoints
Each service has its own endpoint providing detailed status and metadata.
- **GET /api/v1/bisq**
- **GET /api/v1/rgb**
- **GET /api/v1/bitvm**
- **GET /api/v1/changelly**
- **GET /api/v1/stacks**
- **GET /api/v1/lightning**
- **GET /api/v1/liquid**
- **GET /api/v1/rootstock**
- **GET /api/v1/babylon**
- **GET /api/v1/bob**
- **GET /api/v1/merlin**
- **GET /api/v1/botanix**
- **GET /api/v1/b2network**
- **GET /api/v1/citrea**
- **GET /api/v1/bitlayer**
- **GET /api/v1/alpen**
- **GET /api/v1/mezo**
- **GET /api/v1/zulu**
- **GET /api/v1/bison**
- **GET /api/v1/hemi**
- **GET /api/v1/taproot-assets**
- **GET /api/v1/nubit**
- **GET /api/v1/lorenzo**
- **GET /api/v1/prices**

## 3. Data Models

### ServiceStatus
```json
{
  "name": "stacks",
  "status": "active",
  "last_checked": "2024-05-20T10:00:00Z",
  "latency_ms": 65,
  "trust_model": "PoX",
  "risk_level": "Medium",
  "data_availability": "On-chain",
  "settlement": "Bitcoin",
  "bridge_security": "sBTC Bridge",
  "version": "1.0.0",
  "metadata": {
    "block_height": "840000",
    "sbtc_bridge_status": "active"
  }
}
```

### ReserveAsset
```json
{
  "asset": "Stacks (sBTC)",
  "total_supplied": 281.2,
  "total_reserves": 352.5,
  "collateral_ratio": 125.3,
  "status": "Audited"
}
```

### AffiliateInfo
```json
{
  "partner_id": "CONXIAN_GLOBAL",
  "status": "active",
  "commission_rate": 0.15,
  "active_campaigns": 5,
  "total_referrals": 12450
}
```

### MarketingInfo
```json
{
  "channel": "X/Twitter",
  "status": "active",
  "active_offers": ["L2_SUMMER"],
  "reach": 500000
}
```

## 4. Protocol-Specific Endpoints

### POST /api/v1/lightning/invoice
Generates a Lightning invoice.
- **Request Body:** `{ "amount_msat": 10000, "description": "Coffee" }`
- **Response:** `{ "invoice": "lnbc...", "payment_hash": "...", "description": "Coffee", "expiry": 3600 }`

### POST /api/v1/lightning/pay
Sends a Lightning payment.
- **Request Body:** `{ "invoice": "lnbc..." }`
- **Response:** `{ "status": "success", "preimage": "...", "destination": "...", "amount_msat": 10000 }`

### GET /api/v1/stacks/contract/{id}
Retrieves Clarity contract details.

### GET /api/v1/liquid/peg
Retrieves Liquid L-BTC peg and federation status.

### GET /api/v1/rootstock/powpeg
Retrieves Rootstock RBTC Powpeg and hashrate status.

### GET /api/v1/babylon/staking
Retrieves Babylon BTC staking and security metrics.

### GET /api/v1/b2network/status
Retrieves B² Network specific status including sequencer batches.

### GET /api/v1/citrea/proof/{id}
Retrieves Citrea ZK proof details for a specific batch.
