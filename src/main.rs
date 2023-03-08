use bili_batch_unsubcribe::auth::qrcode::QrScanStatus;
use bili_batch_unsubcribe::auth::qrcode::{generate_qrcode_key, verify_qrcode_key};

use bili_batch_unsubcribe::user::me;

use bili_batch_unsubcribe::{Client, Result};

use bili_batch_unsubcribe::user::relation::unsubcribe_users_with_tag;

use reqwest_middleware::ClientBuilder;
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use tracing::info;

use time::UtcOffset;
use tracing_subscriber::fmt::time::OffsetTime;

#[tokio::main]
async fn run() -> Result<()> {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(5);
    let mut client = ClientBuilder::new(reqwest::Client::builder()
        .cookie_store(true)
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36")
        .build()?)
        // Trace HTTP requests. See the tracing crate to make use of these traces.
        // Retry failed requests.
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let qrgen = generate_qrcode_key(&mut client).await?;

    qr2term::print_qr(qrgen.url)?;

    let login_result = qrcode_login(&client, &qrgen.qrcode_key).await;

    match &login_result {
        Some((timestamp, csrf_token)) => {
            info!("logined successfully at {timestamp}, csrf: {csrf_token}")
        }
        None => {
            info!("qrcode expired. exiting");
            std::process::exit(1);
        }
    }

    let (_timestamp, csrf) = login_result.unwrap();

    let me::UserInfo { mid, uname } = me::my_info(&mut client).await?;
    info!("id: {mid}, name: {uname}");

    unsubcribe_users_with_tag(&client, 350356, &csrf).await?;

    Ok(())
}

fn main() -> Result<()> {
    let offset = UtcOffset::current_local_offset().expect("should get local offset!");
    let timer = OffsetTime::new(offset, time::format_description::well_known::Rfc3339);
    let collector = tracing_subscriber::fmt().with_timer(timer);
    collector.init();

    run()
}

async fn qrcode_login(client: &Client, qr_key: &str) -> Option<(u64, String)> {
    use std::time::Duration;
    loop {
        let re = verify_qrcode_key(client, qr_key).await;
        if let Ok(status) = re {
            match status {
                QrScanStatus::Expired => {
                    eprintln!("qrcode expired");
                    return None;
                }
                QrScanStatus::Success(time, csrf) => return Some((time, csrf)),
                QrScanStatus::Unconfirmed => info!("scanned but not confirmed"),
                QrScanStatus::Unscanned => continue,
            }
        }

        tokio::time::sleep(Duration::from_millis(3000)).await;
    }
}
