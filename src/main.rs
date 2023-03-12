use bili_batch_unsubscribe::auth::qrcode::generate_qrcode_key;
use rustc_hash::FxHashMap;

use bili_batch_unsubscribe::user::myself;
use bili_batch_unsubscribe::user::relation::{list_tags, unsubcribe_users_with_tag, Tag};

use bili_batch_unsubscribe::{Client, Result};

use tracing::{error, info};

#[tokio::main]
async fn run() -> Result<()> {
    let client = build_reqwest_client()?;

    let login_result = login_with_qr(&client).await?;
    let (_, csrf) = login_result.unwrap();

    let myself::UserInfo { mid, uname } = myself::my_info(&client).await?;
    info!("id: {mid}, name: {uname}");

    let tags = list_tags(&client).await?;
    let tags_map = FxHashMap::from_iter(tags.iter().map(|Tag { name, tagid, .. }| (name, tagid)));

    let tags_selected = inquire::MultiSelect::new(
        "请选择要批量取关的分组：",
        tags.iter().map(|t| &t.name).collect(),
    )
    .with_help_message("↑↓移动，空格选择，→ 全选，← 全不选, 输入即可搜索")
    .prompt()?;

    for tag_name in tags_selected {
        unsubcribe_users_with_tag(&client, *tags_map[tag_name], &csrf).await?;
    }

    Ok(())
}

fn main() -> Result<()> {
    use time::UtcOffset;
    use tracing_subscriber::fmt::time::OffsetTime;

    let offset = UtcOffset::current_local_offset().expect("should get local offset!");
    let timer = OffsetTime::new(offset, time::format_description::well_known::Rfc3339);
    let collector = tracing_subscriber::fmt().with_timer(timer);
    collector.init();

    run()
}

async fn login_with_qr(client: &Client) -> Result<Option<(u64, String)>> {
    let qrgen = generate_qrcode_key(&client).await?;

    qr2term::print_qr(qrgen.url)?;

    let login_result = check_until_login_qr(&client, &qrgen.qrcode_key).await;

    match login_result {
        Some((timestamp, csrf_token)) => {
            info!("logined successfully at {timestamp}, csrf: {csrf_token}");
            Ok(Some((timestamp, csrf_token)))
        }
        None => {
            error!("qrcode expired. exiting");
            std::process::exit(1);
        }
    }
}

fn build_reqwest_client() -> Result<Client> {
    use reqwest_middleware::ClientBuilder;
    use reqwest_retry::policies::ExponentialBackoff;
    use reqwest_retry::RetryTransientMiddleware;

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(5);
    Ok(ClientBuilder::new(reqwest::Client::builder()
        .cookie_store(true)
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36")
        .build()?)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build())
}

async fn check_until_login_qr(client: &Client, qr_key: &str) -> Option<(u64, String)> {
    use bili_batch_unsubscribe::auth::qrcode::verify_qrcode_key;
    use bili_batch_unsubscribe::auth::qrcode::QrScanStatus;
    use std::time::Duration;
    loop {
        let re = verify_qrcode_key(client, qr_key).await;
        if let Ok(status) = re {
            match status {
                QrScanStatus::Expired => {
                    return None;
                }
                QrScanStatus::Unconfirmed => info!("scanned but not confirmed"),
                QrScanStatus::Unscanned => continue,
                QrScanStatus::Success { timestamp, csrf } => return Some((timestamp, csrf)),
            }
        }

        tokio::time::sleep(Duration::from_millis(3000)).await;
    }
}
