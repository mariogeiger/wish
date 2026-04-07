use reqwest::Client;
use serde::Serialize;

#[derive(Clone)]
pub struct ResendClient {
    client: Client,
    api_key: String,
    from: String,
}

#[derive(Serialize)]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: Vec<&'a str>,
    subject: &'a str,
    html: &'a str,
    text: &'a str,
}

impl ResendClient {
    pub fn new(api_key: String, from: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            from,
        }
    }

    pub async fn send(
        &self,
        to: &str,
        subject: &str,
        html: &str,
        text: &str,
    ) -> Result<(), String> {
        let body = SendEmailRequest {
            from: &self.from,
            to: vec![to],
            subject,
            html,
            text,
        };

        let resp = self
            .client
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(format!("Resend API error {status}: {body}"))
        }
    }
}
