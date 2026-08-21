use crate::config::settings::NotificationConfig;
use crate::swarm::types::*;
use tracing::info;

pub struct NotificationManager {
    config: NotificationConfig,
}

impl NotificationManager {
    pub fn new(config: &NotificationConfig) -> Self {
        NotificationManager {
            config: config.clone(),
        }
    }

    pub async fn notify_issue_detected(&self, issue: &Issue) {
        let message = format!(
            "[AAS] {} detected by {}: {}",
            issue.severity, issue.agent_name, issue.title
        );
        info!("Notification: {}", message);
        self.send(&message).await;
    }

    pub async fn notify_action_completed(&self, action: &Action, result: &ActionResult) {
        if !self.config.triggers.fixed_issues {
            return;
        }
        let status = if result.success { "✅" } else { "❌" };
        let message = format!(
            "{} Action '{}' by {}: {}",
            status, action.description, action.agent_name, result.output
        );
        info!("Notification: {}", message);
        self.send(&message).await;
    }

    pub async fn notify_escalation(&self, agent: &str, issue: &Issue, reason: &str) {
        if !self.config.triggers.escalations {
            return;
        }
        let message = format!(
            "🚨 Escalation from {}: {} - {} ({})",
            agent, issue.title, reason, issue.severity
        );
        info!("Notification: {}", message);
        self.send(&message).await;
    }

    pub async fn notify_prediction(&self, prediction: &Prediction) {
        if !self.config.triggers.predictions {
            return;
        }
        let message = format!(
            "🔮 Prediction from {}: {} ({}% confidence)",
            prediction.agent_name,
            prediction.predicted_issue,
            (prediction.confidence * 100.0) as u32
        );
        info!("Notification: {}", message);
        self.send(&message).await;
    }

    pub async fn notify_error(&self, agent: &str, error: &str) {
        if !self.config.triggers.errors {
            return;
        }
        let message = format!("⚠️ Agent '{}' error: {}", agent, error);
        info!("Notification: {}", message);
        self.send(&message).await;
    }

    async fn send(&self, message: &str) {
        if self.config.channels.contains(&"slack".to_string()) {
            if let Some(ref slack) = self.config.slack {
                if let (Some(ref token), Some(ref channel)) = (&slack.bot_token, &slack.channel) {
                    self.send_slack(token, channel, message).await;
                }
            }
        }

        if self.config.channels.contains(&"email".to_string()) {
            if let Some(ref email) = self.config.email {
                if let Some(ref addr) = email.address {
                    info!("[EMAIL to {}]: {}", addr, message);
                }
            }
        }
    }

    async fn send_slack(&self, token: &str, channel: &str, message: &str) {
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "channel": channel,
            "text": message,
        });

        let _ = client
            .post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", token))
            .json(&payload)
            .send()
            .await;
    }
}
