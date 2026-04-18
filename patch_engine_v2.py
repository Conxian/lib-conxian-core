import sys

content = open('gateway/src/engine/mod.rs').read()

old_init_start = """    fn initialize_services(&self) {
        let mut statuses = self.service_statuses.write().unwrap();
        let services = vec!["""

new_init_start = """    fn initialize_services(&self) {
        let mut statuses = self.service_statuses.write().unwrap();

        // CON-501: Remove static seeds from production mainnet
        let is_mainnet = remediation::is_production_mainnet();

        let services = if is_mainnet {
            // Production paths must fetch data from real RPCs/Nodes
            // Initializing with zero/placeholder to ensure discovery before functional use
            vec![
                ("stacks", 0, "PoX", "On-chain", "Bitcoin", "sBTC Bridge", 0.0),
                ("lightning", 0, "State Channels", "Off-chain", "Bitcoin", "P2P", 0.0),
                ("liquid", 0, "Federation", "Sidechain", "Bitcoin", "Powpeg", 0.0),
                ("rootstock", 0, "Merge-mined", "Sidechain", "Bitcoin", "Powpeg", 0.0),
                ("bisq", 0, "P2P", "Off-chain", "Bitcoin", "Atomic", 0.0),
                ("rgb", 0, "Client-side", "Off-chain", "Bitcoin", "N/A", 0.0),
                ("bitvm", 0, "Optimistic", "On-chain", "Bitcoin", "BitVM", 0.0),
                ("babylon", 0, "Staking", "On-chain", "Bitcoin", "Staking", 0.0),
                ("core-dao", 0, "Satoshi Plus", "Sidechain", "Bitcoin", "Relayer", 0.0),
                ("lorenzo", 0, "Staking", "On-chain", "Bitcoin", "Staking", 0.0),
                ("hemi", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
                ("bob", 0, "Optimistic", "Rollup", "Bitcoin", "Optimistic", 0.0),
                ("merlin", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
                ("mezo", 0, "Economic Layer", "On-chain", "Bitcoin", "tBTC", 0.0),
                ("nubit", 0, "DA", "On-chain", "Bitcoin", "N/A", 0.0),
                ("bison", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
                ("zulu", 0, "Multi-layer", "On-chain", "Bitcoin", "N/A", 0.0),
                ("botanix", 0, "Spiderchain", "Sidechain", "Bitcoin", "Spiderchain", 0.0),
                ("bitlayer", 0, "Optimistic", "Rollup", "Bitcoin", "BitVM", 0.0),
                ("alpen", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
                ("taproot-assets", 0, "Client-side", "Off-chain", "Bitcoin", "N/A", 0.0),
                ("bitvm2", 0, "ZK-Fraud Proofs", "On-chain", "Bitcoin", "BitVM2", 0.0),
            ]
        } else {
            vec!["""

old_init_end = """            (
                "bitvm2",
                85,
                "ZK-Fraud Proofs",
                "On-chain",
                "Bitcoin",
                "BitVM2",
                15000000.0,
            ),
        ];"""

new_init_end = """            (
                "bitvm2",
                85,
                "ZK-Fraud Proofs",
                "On-chain",
                "Bitcoin",
                "BitVM2",
                15000000.0,
            ),
        ]};"""

content = content.replace(old_init_start, new_init_start)
content = content.replace(old_init_end, new_init_end)

with open('gateway/src/engine/mod.rs', 'w') as f:
    f.write(content)
