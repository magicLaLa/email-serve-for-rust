use axum::{body::Body, extract::Extension, http::Request, Router, Server};
use email::{Email, SmtpEmail};
use std::{env, net::SocketAddr};
use time::{format_description, UtcOffset};
use tokio::sync::oneshot;
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{event, Level, Span};
use tracing_subscriber::fmt::time::OffsetTime;

use serve::Email_Serve;

/// 默认端口号
const PORT: u16 = 1919;
/// 默认账户
const USER_NAME: &str = "";
const PASS_WORD: &str = "";

#[tokio::main]
async fn main() {
    // 修改为本地时间
    let format = "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]";
    let local_time = OffsetTime::new(
        UtcOffset::from_hms(8, 0, 0).unwrap(),
        format_description::parse(format).unwrap(),
    );
    tracing_subscriber::fmt()
        .with_timer(local_time)
        .json()
        .with_current_span(true)
        .init();

    let port = match env::var("PORT") {
        Ok(op) => op.parse::<u16>().unwrap_or(PORT),
        Err(_) => PORT,
    };

    let user_name = match env::var("USER") {
        Ok(op) => op.parse::<String>().unwrap_or_else(|_| USER_NAME.to_string()),
        Err(_) => USER_NAME.to_string(),
    };

    let pasword = match env::var("PASS_WORD") {
        Ok(op) => op.parse::<String>().unwrap_or_else(|_| PASS_WORD.to_string()),
        Err(_) => PASS_WORD.to_string(),
    };

    if user_name.is_empty() || pasword.is_empty() {
        panic!("USER or PASS_WORD is Required !");
    }



    let smtp_serve = if cfg!(feature = "QQ") {
        "smtp.qq.com"
    } else if cfg!(feature = "163") {
        "smtp.163.com"
    } else {
        "smtp.gmail.com"
    };

    let mailer = match Email::new(
        smtp_serve.to_string(),
        user_name,
        pasword,
    ).async_builder() {
        Ok(r) => r,
        Err(e) => {
            panic!("email_serv is err: {}", e);
        },
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let app = Router::new()
        .nest("/email", Email_Serve())
        .layer(Extension(mailer))
        .layer(
            // 自定义处理请求日志
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(|request: &Request<Body>, _span: &Span| {
                    event!(target:"tower_http::trace::on_request", Level::INFO, "started  {}", request.uri().path());
                })
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        );

    let serve = Server::bind(&addr).serve(app.into_make_service());
    let (tx, rx) = oneshot::channel::<()>();
    let graceful = serve.with_graceful_shutdown(async {
        rx.await.ok();
    });

    println!("Listening on http://{}", addr);

    tokio::task::spawn(async {
        tokio::signal::ctrl_c().await.expect("exit listen fail");
        let _ = tx.send(());
    });

    if let Err(e) = graceful.await {
        eprintln!("server err: {}", e);
    }
}
