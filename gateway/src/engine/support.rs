use serde::{Deserialize, Serialize};
use chrono::{Datelike, DateTime, Utc};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum SupportChannel {
    Public,
    Internal,
    Community,
}

impl SupportChannel {
    pub fn to_linear_labels(&self) -> Vec<String> {
        let mut labels = vec!["Support".to_string()];
        match self {
            SupportChannel::Public => labels.push("Support-Public".to_string()),
            SupportChannel::Internal => labels.push("Support-Internal".to_string()),
            SupportChannel::Community => {
                labels.push("Support-Community".to_string());
                labels.push("Publish-Candidate".to_string());
            }
        }
        labels
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SupportTicket {
    pub token: String, // SUP-YYYY-MM-DD-####
    pub channel: SupportChannel,
    pub sender_domain: String,
    pub normalized_subject: String,
    pub timestamp: DateTime<Utc>,
    pub message_id: String,
    pub publish_candidate: bool,
    pub status: String, // "Triage"
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SupportConfig {
    pub imap_host: String,
    pub imap_port: u16,
    pub poll_interval_secs: u64,
    pub channel_mappings: HashMap<String, SupportChannel>,
}

impl Default for SupportConfig {
    fn default() -> Self {
        let mut mappings = HashMap::new();
        mappings.insert("support@conxian-labs.com".to_string(), SupportChannel::Public);
        mappings.insert("info@conxian-labs.com".to_string(), SupportChannel::Internal);
        mappings.insert("admin@conxian-labs.com".to_string(), SupportChannel::Internal);
        mappings.insert("community@conxian-labs.com".to_string(), SupportChannel::Community);
        mappings.insert("builders@conxian-labs.com".to_string(), SupportChannel::Community);

        Self {
            imap_host: "mail.privateemail.com".to_string(),
            imap_port: 993,
            poll_interval_secs: 300,
            channel_mappings: mappings,
        }
    }
}

pub struct SupportIntake {
    pub config: SupportConfig,
    sequence: AtomicU64,
}

impl SupportIntake {
    pub fn new(config: SupportConfig) -> Self {
        Self {
            config,
            sequence: AtomicU64::new(1),
        }
    }

    pub fn generate_token(&self, timestamp: DateTime<Utc>) -> String {
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        format!(
            "SUP-{:04}-{:02}-{:02}-{:04}",
            timestamp.year(),
            timestamp.month(),
            timestamp.day(),
            seq % 10000
        )
    }

    pub fn classify_recipient(&self, recipient: &str) -> SupportChannel {
        self.config
            .channel_mappings
            .get(recipient)
            .cloned()
            .unwrap_or(SupportChannel::Public)
    }

    pub fn sanitize_metadata(&self, subject: &str, sender_email: &str) -> (String, String) {
        let sanitized_subject = self.redact_pii(subject);
        let sender_domain = sender_email
            .split('@')
            .collect::<Vec<&str>>()
            .get(1)
            .cloned()
            .unwrap_or("unknown")
            .to_string();

        (sanitized_subject, sender_domain)
    }

    fn redact_pii(&self, input: &str) -> String {
        let mut result = input.to_string();
        if let Some(at_idx) = result.find('@') {
            if let Some(start_idx) = result[..at_idx].rfind(' ') {
                 result.replace_range(start_idx + 1..at_idx + 1, "[REDACTED]@");
            } else {
                 result.replace_range(0..at_idx + 1, "[REDACTED]@");
            }
        }
        result
    }

    pub fn process_inbound_metadata(
        &self,
        recipient: &str,
        sender: &str,
        subject: &str,
        message_id: &str,
        timestamp: DateTime<Utc>,
    ) -> SupportTicket {
        let channel = self.classify_recipient(recipient);
        let (sanitized_subject, sender_domain) = self.sanitize_metadata(subject, sender);
        let token = self.generate_token(timestamp);
        let publish_candidate = channel == SupportChannel::Community;

        SupportTicket {
            token,
            channel,
            sender_domain,
            normalized_subject: sanitized_subject,
            timestamp,
            message_id: message_id.to_string(),
            publish_candidate,
            status: "Triage".to_string(),
        }
    }

    pub fn prepare_linear_issue(&self, ticket: &SupportTicket) -> serde_json::Value {
        serde_json::json!({
            "title": format!("[{}] {}", ticket.token, ticket.normalized_subject),
            "description": format!(
                "**Ticket:** {}\n**Channel:** {:?}\n**Sender Domain:** {}\n**Message ID:** {}\n**Ingress:** {}\n\n*Note: Raw email content remains in PrivateEmail for security.*",
                ticket.token, ticket.channel, ticket.sender_domain, ticket.message_id, ticket.timestamp
            ),
            "labels": ticket.channel.to_linear_labels(),
            "state": "Triage"
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_token_generation() {
        let intake = SupportIntake::new(SupportConfig::default());
        let ts = Utc.with_ymd_and_hms(2026, 4, 11, 0, 0, 0).unwrap();
        let token = intake.generate_token(ts);
        assert!(token.starts_with("SUP-2026-04-11-0001"));
    }

    #[test]
    fn test_classification() {
        let intake = SupportIntake::new(SupportConfig::default());
        assert_eq!(intake.classify_recipient("support@conxian-labs.com"), SupportChannel::Public);
        assert_eq!(intake.classify_recipient("community@conxian-labs.com"), SupportChannel::Community);
        assert_eq!(intake.classify_recipient("unknown@conxian-labs.com"), SupportChannel::Public);
    }

    #[test]
    fn test_sanitization() {
        let intake = SupportIntake::new(SupportConfig::default());
        let (subject, domain) = intake.sanitize_metadata("Issue from user@example.com", "sender@example.com");
        assert!(subject.contains("[REDACTED]"));
        assert_eq!(domain, "example.com");
    }

    #[test]
    fn test_linear_labels() {
        assert!(SupportChannel::Public.to_linear_labels().contains(&"Support-Public".to_string()));
        assert!(SupportChannel::Community.to_linear_labels().contains(&"Publish-Candidate".to_string()));
    }
}

#[cfg(test)]
mod verification_tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn verify_end_to_end_processing() {
        let intake = SupportIntake::new(SupportConfig::default());
        let ts = Utc.with_ymd_and_hms(2026, 4, 11, 10, 30, 0).unwrap();

        // Mock inbound email metadata
        let recipient = "community@conxian-labs.com";
        let sender = "builder@dev.net";
        let subject = "Question about Gateway v2";
        let message_id = "<msg-123@dev.net>";

        let ticket = intake.process_inbound_metadata(recipient, sender, subject, message_id, ts);

        assert_eq!(ticket.channel, SupportChannel::Community);
        assert_eq!(ticket.sender_domain, "dev.net");
        assert!(ticket.token.contains("SUP-2026-04-11"));
        assert!(ticket.publish_candidate);
        assert_eq!(ticket.status, "Triage");

        let linear_issue = intake.prepare_linear_issue(&ticket);
        let labels = linear_issue["labels"].as_array().unwrap();
        assert!(labels.contains(&serde_json::json!("Support-Community")));
        assert!(labels.contains(&serde_json::json!("Publish-Candidate")));
        assert!(linear_issue["title"].as_str().unwrap().contains(&ticket.token));
    }

    #[test]
    fn verify_internal_routing() {
        let intake = SupportIntake::new(SupportConfig::default());
        let ts = Utc::now();

        let recipient = "admin@conxian-labs.com";
        let ticket = intake.process_inbound_metadata(recipient, "admin@internal.com", "System Alert", "id-1", ts);

        assert_eq!(ticket.channel, SupportChannel::Internal);
        assert!(!ticket.publish_candidate);

        let linear_issue = intake.prepare_linear_issue(&ticket);
        let labels = linear_issue["labels"].as_array().unwrap();
        assert!(labels.contains(&serde_json::json!("Support-Internal")));
        assert!(!labels.contains(&serde_json::json!("Publish-Candidate")));
    }
}
