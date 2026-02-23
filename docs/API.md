# Conxian Gateway API Reference

The Conxian Gateway API is accessible under the `/api/v1` prefix.

## 1. System Endpoints

### GET /api/v1/health
Returns the health status of the gateway.
- **Response:** `{ "status": "healthy" }`

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
- **Response:** `{ "status": "compliant", "last_audit": "...", "rules_active": ["KYC", "AML", "NetworkIntegrity"] }`

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

## 4. Protocol-Specific Endpoints (Phase 4)

### POST /api/v1/lightning/invoice
Generates a Lightning invoice.
- **Request Body:** `{ "amount_msat": 10000, "description": "Coffee" }`
- **Response:** `{ "invoice": "lnbc...", "payment_hash": "..." }`

### POST /api/v1/lightning/pay
Sends a Lightning payment.
- **Request Body:** `{ "invoice": "lnbc..." }`
- **Response:** `{ "status": "success", "preimage": "..." }`

### GET /api/v1/stacks/contract/{id}
Retrieves Clarity contract details.
- **Response:** `{ "contract_id": "...", "source_code": "...", "abi": "..." }`

### GET /api/v1/rgb/contract/{id}
Retrieves RGB contract details and state.
- **Response:** `{ "contract_id": "...", "schema": "...", "state": "..." }`

### GET /api/v1/bitvm/proof/{id}
Retrieves BitVM fraud proof details and status.
- **Response:** `{ "proof_id": "...", "status": "...", "verifier_count": 5 }`

### PriceInfo
```json
{
  "asset": "BTC",
  "price_usd": 65000.0,
  "last_updated": "2024-05-20T10:00:00Z",
  "source": "Conxian Oracle"
}
```


### ComplianceStatus
```json
{
  "status": "compliant",
  "last_audit": "2024-05-20T10:00:00Z",
  "rules_active": ["KYC", "AML", "NetworkIntegrity"],
  "risk_score": 15
}
```

### GET /api/v1/changelly/rate
Returns a simulated exchange rate between two assets.
- **Query Parameters:** `from`, `to`
- **Response:** `{ "from": "BTC", "to": "USD", "rate": 65000.0, "timestamp": "..." }`
