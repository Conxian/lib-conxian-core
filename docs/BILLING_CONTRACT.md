# Shared billing configuration contract, version 1

Tracks [Business #1343](https://github.com/Conxian/conxian-business/issues/1343)
and [#1390](https://github.com/Conxian/conxian-business/issues/1390).

`billing::billing_catalog()` and `spec/billing_conformance.v1.json` describe
the existing configured schedules. Core owns the offline definitions; Gateway
owns service-charge calculation and Nexus owns tenant/payment/quota enforcement.
The catalog is configuration only. It is not a current offer, paid entitlement,
collection receipt, provider support claim or permission to execute.

| Product | Unit and scope | Implementation boundary |
| --- | --- | --- |
| Gateway managed service | USD cents; existing base and usage rates, 30-day period, volume discount over base plus usage | `billing::gateway` constants; Gateway keeps `compute_mrr` |
| Nexus signature access | Signature quota, satoshi upgrade amount and invoice expiry | `billing::nexus` definitions; Nexus keeps its existing public tier type and tenant enforcement |
| Market settlement | ADR-004 settlement fee, distinct from service charges and signature quotas | Existing `fee` module; no second fee calculator |
| Website acquisition | Availability and offer presentation | Must not publish a purchasable subscription until the payment/entitlement flow exists |

Gateway's volume threshold does not establish Nexus Enterprise status. Nexus
Enterprise's `enterprise_fee_cap()` is the existing ADR-004 percentage-to-floor
eligibility flag, not a service-charge discount or evidence that a subscription
is active or paid. No cross-product entitlement mapping is implemented by this
contract. Runtime code must authenticate the tenant and verify the actual
product entitlement before applying payment-dependent privileges.

x402 is a payment protocol, not an asset or accounting currency. A future
payment integration must identify the actual asset, network and atomic unit,
then verify its payment/settlement outcome separately. No currency conversion,
treasury transfer, subscription renewal period, new price or native Conxian
token is introduced here.

## Compatibility and validation

Consumers pin the full reviewed Core commit SHA. Existing rates and tier wire
values are preserved; changing a schedule requires a new reviewed policy
revision and matching artifact. The catalog's schema version describes its
shape, while `policy_id` and the pinned Core revision identify its configuration.

The development Core SDK feature surface pins SDK 2.1.0 commit
`cb6b9c69752b7082d71b0ea7431a40a498ae8a6e`, which carries the reviewed
provider-policy/key-parameter corrections absent from the former development
pin. Features remain opt-in; no simulator or signing feature is enabled by the
billing contract. The separately published SDK =2.0.17 companion adapter keeps
its explicit compatibility boundary and tests. Updating that published-version
adapter and qualifying any release remain separate work.

`validate_billing_catalog_json` accepts only a bounded document matching this
exact configuration. It rejects unknown fields, missing fields, changed prices,
units, authority claims and versions. This detects configuration drift; it does
not authenticate the source of a matching JSON document. The artifact and Rust
definitions are checked together in Core tests. Downstream conformance tests
check actual calculations and the preserved public Nexus tier behavior.

```rust
use lib_conxian_core::billing::{billing_catalog, BILLING_CATALOG_JSON,
    validate_billing_catalog_json};

let catalog = validate_billing_catalog_json(BILLING_CATALOG_JSON)?;
assert_eq!(catalog, billing_catalog());
assert!(!catalog.confers_entitlement);
# Ok::<(), &'static str>(())
```

Payment webhook signatures/replay handling, durable entitlement/key lifecycle,
real metering, invoice/receipt integration and reviewed rollout through
dev -> staged -> main remain separate acceptance work. This additive contract
does not close the parent issues or qualify production capabilities.
