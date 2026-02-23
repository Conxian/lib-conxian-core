# Update PRD
with open('docs/PRD.md', 'r') as f:
    prd = f.read()

if 'Citrea' not in prd:
    prd = prd.replace(
        'Features block height and proof verification tracking.',
        'Features block height and proof verification tracking.\n- **Citrea:** The first ZK-Rollup on Bitcoin (ZK). Features TVL and ZK-proof verification monitoring.\n- **Bitlayer:** The first Bitcoin Layer 2 based on BitVM (Optimistic). Features TVL and BitVM bridge security monitoring.'
    )
    with open('docs/PRD.md', 'w') as f:
        f.write(prd)

# Update API
with open('docs/API.md', 'r') as f:
    api = f.read()

if 'citrea' not in api:
    api = api.replace(
        '- **GET /api/v1/b2network**',
        '- **GET /api/v1/b2network**\n- **GET /api/v1/citrea**\n- **GET /api/v1/bitlayer**\n- **GET /api/v1/prices**'
    )
    api += """
### PriceInfo
```json
{
  "asset": "BTC",
  "price_usd": 65000.0,
  "last_updated": "2024-05-20T10:00:00Z",
  "source": "Conxian Oracle"
}
```
"""
    with open('docs/API.md', 'w') as f:
        f.write(api)

# Update ENHANCEMENTS
with open('docs/ENHANCEMENTS.md', 'r') as f:
    enh = f.read()

if 'Citrea' not in enh:
    enh = enh.replace(
        '- **New Layer Integration:** Added support for Babylon, BOB, Merlin, Botanix, and B² Network.',
        '- **New Layer Integration:** Added support for Babylon, BOB, Merlin, Botanix, B² Network, Citrea, and Bitlayer.\n- **Unified Price Feed:** Integrated a simulated real-time price feed for core assets (BTC, STX).'
    )
    with open('docs/ENHANCEMENTS.md', 'w') as f:
        f.write(enh)
