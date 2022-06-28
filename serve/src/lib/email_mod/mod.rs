use axum::{body::Bytes, extract::Extension, http::StatusCode, routing::post, Json, Router};
use email::{
    message::{header, MultiPart, SinglePart},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{event, Level};


#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "config", rename_all = "camelCase")]
enum MessageConfig<'a> {
  Html {
    form: &'a str,
    to: &'a str,
    reply_to: Option<&'a str>,
    subject: &'a str,
    spare: String,
    title: &'a str,
    content: &'a str,
  },
  Text {
    form: &'a str,
    to: &'a str,
    reply_to: Option<&'a str>,
    subject: &'a str,
    body: String,
  }
}

fn build_message_html<'a>(
  form: &'a str,
  to: &'a str,
  reply_to: Option<&'a str>,
  subject: &'a str,
  spare: String,
  title: &'a str,
  content: &'a str,
) -> Message {
    let html = format!(
      r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>{}</title>
        </head>
        <body>
            <div style="display: flex; flex-direction: column; align-items: center;">
                <h2 style="font-family: Arial, Helvetica, sans-serif;">{}</h2>
                <h4 style="font-family: Arial, Helvetica, sans-serif;">{}</h4>
            </div>
        </body>
        </html>
      "#,
      title,
      title,
      content,
    );
    if let Some(reply_to) = reply_to {
      Message::builder()
        .from(form.parse().unwrap())
        .to(to.parse().unwrap())
        .reply_to(reply_to.parse().unwrap())
        .subject(subject)
        .multipart(
            MultiPart::alternative() // This is composed of two parts.
                .singlepart(
                    // 这条不会，html 失败的话这条会作为纯文本发送
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(spare), // Every message should have a plain text fallback.
                )
                .singlepart(
                    // 这条会被发送
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(html),
                ),
        )
        .expect("failed to build email")
    } else {
      Message::builder()
        .from(form.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .multipart(
            MultiPart::alternative() // This is composed of two parts.
                .singlepart(
                    // 这条不会，html 失败的话这条会作为纯文本发送
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(spare), // Every message should have a plain text fallback.
                )
                .singlepart(
                    // 这条会被发送
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(html),
                ),
        )
        .expect("failed to build email")
    }
}

fn build_message<'a>(
  form: &'a str,
  to: &'a str,
  reply_to: Option<&'a str>,
  subject: &'a str,
  body: String,
) -> Message {
    if let Some(reply_to) = reply_to {
        Message::builder()
            .from(form.parse().unwrap())
            .reply_to(reply_to.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(subject)
            .body(body)
            .unwrap()
    } else {
        Message::builder()
            .from(form.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(subject)
            .body(body)
            .unwrap()
    }
}

async fn send_email(
    buf: Bytes,
    mailer: Extension<AsyncSmtpTransport<Tokio1Executor>>,
) -> (StatusCode, Json<Value>) {
    match serde_json::from_slice::<MessageConfig>(&buf) {
        Ok(res) => {
            let message = match res {
                MessageConfig::Html { form, to, reply_to, subject, spare, title, content } => {
                  build_message_html(form, to, reply_to, subject, spare, title, content)
                },
                MessageConfig::Text { form, to, reply_to, subject, body } => {
                  build_message(form, to, reply_to, subject, body)
                },
            };
            return match mailer.send(message).await {
                Ok(_) => (
                    StatusCode::OK,
                    Json(json!({
                      "message": "成功"
                    })),
                ),
                Err(e) => (
                    StatusCode::OK,
                    Json(json!({
                      "err": e.to_string(),
                    })),
                ),
            };
        }
        Err(e) => {
            event!(target:"tower_http::trace::EmailMessage", Level::WARN, "{:?}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "err": e.to_string()
                })),
            )
        }
    }
}

pub fn router() -> Router {
    Router::new().route("/send", post(send_email))
}
