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

## 3. Canonical chain-risk metadata (CORE-007)

The six chain-risk dimensions are a separate, versioned protocol metadata contract. They are not
the operational metrics above and are not live Gateway telemetry.

| Field | Unit and polarity | Static-core owner | Live consumer/owner |
| --- | --- | --- | --- |
| `data_availability_score` | Unitless strength, `0..=100`; higher is stronger assessed availability | `lib-conxian-core` schema and bounds | Nexus evidence and Gateway policy |
| `settlement_score` | Unitless strength, `0..=100`; higher is stronger assessed settlement assurance | Core schema and bounds | Nexus evidence and Gateway policy |
| `bridge_score` | Unitless strength, `0..=100`; higher is stronger assessed bridge controls | Core schema and bounds | Nexus evidence and Gateway policy |
| `exit_mechanism_score` | Unitless strength, `0..=100`; higher is stronger assessed exit guarantees | Core schema and bounds | Nexus evidence and Gateway policy |
| `operator_dependency_score` | Unitless strength, `0..=100`; higher means greater operator independence/resilience | Core schema and polarity | Nexus observations and Gateway policy |
| `decentralization_score` | Unitless strength, `0..=100`; higher is stronger assessed decentralization | Core schema and bounds | Nexus evidence and Gateway policy |

`0` is a valid assessed score. `unknown` and `not_assessed` are explicit wire states and must not
be represented by a missing metric or a zero. Assessed and partially assessed profiles require
typed evidence references (`specification`, `audit`, `research`, or `observation`); governance/change
references are separate and cannot satisfy that requirement. Not-assessed profiles require a
governance/change reference, effective date, and rationale, but may have no evidence. Schema-v1
JSON decoding rejects semantic violations and unknown/typo fields; additive fields require a schema
decision/version bump.

The schema-v1 artifact and review process are documented in
[`docs/architecture/RISK_PROFILES.md`](architecture/RISK_PROFILES.md). Profile changes must update
the artifact, profile revision, set version when applicable, evidence/change reference, tests,
documentation, and release notes together. Core does not fetch these metrics or turn them into a
route; Gateway and Nexus own those runtime decisions.
The operational metrics above remain separate from this static contract. A metric refresh, market
observation, or Gateway risk calculation must not be serialized as an approved Core risk score;
unknown or stale metadata must fail closed downstream.

---
*Last Updated: 2026-05-27*
