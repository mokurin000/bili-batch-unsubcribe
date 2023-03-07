use std::time::Duration;

use bili_batch_unsubcribe::auth::QrScanStatus;
use bili_batch_unsubcribe::auth::{generate_qrcode_key, verify_qrcode_key};

use bili_batch_unsubcribe::{Client, Result};

use tracing::{info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut client = Client::builder()
        .cookie_store(true)
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36")
        .build()?;

    let qrgen = generate_qrcode_key(&mut client).await?;

    qr2term::print_qr(qrgen.url)?;

    let login_result = login(&mut client, &qrgen.qrcode_key).await;

    match login_result {
        Some(login_timestamp) => info!("logined successfully at {login_timestamp}"),
        None => {
            info!("qrcode expired. exiting");
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn login(client: &mut Client, qr_key: &str) -> Option<u64> {
    loop {
        let re = verify_qrcode_key(client, qr_key).await;
        if let Ok(status) = re {
            match status {
                QrScanStatus::Expired => {
                    eprintln!("qrcode expired");
                    return None;
                }
                QrScanStatus::Success(time) => return Some(time),
                QrScanStatus::Unconfirmed => info!("scanned but not confirmed"),
                QrScanStatus::Unscanned => continue,
            }
        }

        tokio::time::sleep(Duration::from_millis(5000)).await;
    }
}
