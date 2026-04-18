import sys

content = open('gateway/src/engine/mod.rs').read()

# Add imports
if 'use crate::engine::support::{SupportConfig, SupportIntake};' not in content:
    content = content.replace('use std::sync::{Arc, RwLock};', 'use std::sync::{Arc, RwLock};\nuse crate::engine::support::{SupportConfig, SupportIntake};')

# Add field to Engine struct
if 'pub support_intake: Arc<SupportIntake>,' not in content:
    content = content.replace('pub start_time: DateTime<Utc>,', 'pub start_time: DateTime<Utc>,\n    pub support_intake: Arc<SupportIntake>,')

# Initialize field in Engine::new
if 'support_intake: Arc::new(SupportIntake::new(SupportConfig::default())),' not in content:
    content = content.replace('start_time: Utc::now(),', 'start_time: Utc::now(),\n            support_intake: Arc::new(SupportIntake::new(SupportConfig::default())),')

# Add poll_support method
poll_support_method = """
    pub async fn poll_support(engine: Arc<Engine>) {
        tokio::spawn(async move {
            loop {
                // In a real implementation, this would connect to IMAP
                // For now, we simulate periodic polling of the support mailbox
                log::info!("Polling support mailbox for new tickets...");

                // Simulate finding an email
                let ts = Utc::now();
                let ticket = engine.support_intake.process_inbound_metadata(
                    "support@conxian-labs.com",
                    "user@external.com",
                    "Assistance required with MuSig2",
                    "<sim-123@external.com>",
                    ts,
                );

                log::info!("Generated support ticket: {}", ticket.token);

                tokio::time::sleep(std::time::Duration::from_secs(engine.support_intake.config.poll_interval_secs)).await;
            }
        });
    }
"""

if 'pub async fn poll_support' not in content:
    # Insert before the last closing brace of impl Engine
    last_brace_idx = content.rfind('}')
    content = content[:last_brace_idx] + poll_support_method + content[last_brace_idx:]

with open('gateway/src/engine/mod.rs', 'w') as f:
    f.write(content)
