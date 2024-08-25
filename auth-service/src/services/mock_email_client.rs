use crate::domain::{email::Email, email_client::EmailClient};
use color_eyre::eyre::Result;

#[derive(Default)]
pub struct MockEmailClient;

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(&self, recipient: &Email, subject: &str, content: &str) -> Result<()> {
        // Our mock email client will simply log the recipient, subject, and content to standard output
        tracing::info!(
            "Sending email to {:?} with subject: {:?} and content: {:?}",
            recipient.as_ref(),
            subject,
            content
        );

        Ok(())
    }
}
