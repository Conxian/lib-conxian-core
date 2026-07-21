# Metric Specifications, Ownership, and Data Contracts (CON-682)

## 1. Metric Definitions & Formulas

### $C_R$ (Cost of Reproduction)
- **Formula:** $C_R = \sum(Architectural Complexity + IP Moat + Integration Depth)$
- **Goal:** Maximize structural defensibility.
- **Owner:** Engineering Lead (@botshelomokoka)
- **Data Source:** System Graph Analysis + PR Audit
- **Refresh Cadence:** Per Milestone Release

### $O_C$ (Opportunity Cost)
- **Formula:** $O_C = \sum(Manual Work Hours + Context Switch Tax)$
- **Goal:** Drive toward zero ($O_C \to 0$).
- **Owner:** Founder / Product Owner (@botshelomokoka)
- **Data Source:** Time-tracking logs + Linear velocity drift
- **Refresh Cadence:** Weekly

### $V_X$ (Execution Velocity)
- **Formula:** $V_X = \frac{\text{Completed Story Points}}{\text{Human Hours}} \times \text{AI Leverage Factor}$
- **Goal:** Maximize leverage via agentic tools.
- **Owner:** Agentic Infrastructure (@botshelomokoka / Charlie)
- **Data Source:** Linear cycle metrics + Git activity
- **Refresh Cadence:** Per Cycle

### $A_S$ (System Autonomy)
- **Formula:** $A_S = \frac{\text{Automated Events}}{\text{Total Operational Events}}$
- **Goal:** Maximize programmatic independence.
- **Owner:** Infrastructure / SRE
- **Data Source:** standalone Gateway Telemetry + BOS logs
- **Refresh Cadence:** Monthly

### $N_E$ (Network Effects)
- **Formula:** $N_E = (\text{Active Nodes} \times \text{Enterprise Partners})^{2}$
- **Goal:** Exponential scale multiplier.
- **Owner:** Growth & Partnerships (@botshelomokoka)
- **Data Source:** On-chain Node Registry + ERP Integration counts
- **Refresh Cadence:** Quarterly

## 2. Data Contracts & Dependencies

- **Contract CX-001:** All operational data for $A_S$ must be exposed via the standalone Gateway `/api/v1/metrics` endpoint in Prometheus-compatible format.
- **Contract CX-002:** Velocity tracking requires all development tasks to be tagged with `AI-Assisted` or `Manual` in Linear for $V_X$ calculation.
- **Dependency D-001:** $N_E$ reporting depends on successful ERP sync completion as defined in the Institutional Alignment row.

## 3. Relationship to canonical chain risk profiles

The metrics in this document describe operational and strategic reporting; they
are not the static chain-family risk-profile schema. A metric refresh, market
observation, or Gateway risk calculation must not be serialized as an approved
Core risk score. Canonical profiles use the separately versioned
`control_model::risk_profile` contract, public evidence, and governance
references. Nexus owns live proof/finality/freshness observations and Gateway
owns routing and policy decisions. Unknown or stale metadata must fail closed
downstream.

---
*Last Updated: 2026-05-27*
