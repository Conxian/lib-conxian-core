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
