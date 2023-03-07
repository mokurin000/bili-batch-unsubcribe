use std::time::Duration;

use serde::Deserialize;
use tracing::info;

use crate::Client;
use crate::Result;

#[derive(Deserialize)]
struct Response {
    code: i32,
    message: String,
    ttl: i32,
    data: Vec<FollowingDetail>,
}

#[derive(Deserialize)]
pub struct FollowingDetail {
    pub mid: u64,
}

pub async fn unsubcribe_users_with_tag(client: &Client, tag: i32, csrf: &str) -> Result<()> {
    let count = client
        .get(format!(
            "https://api.bilibili.com/x/relation/tag?tagid={tag}"
        ))
        .send()
        .await?
        .json::<Response>()
        .await?
        .data
        .len();
    for pn in 1..=((count as f64) / 50.0).ceil() as i32 {
        let resp = client
            .get("https://api.bilibili.com/x/relation/tag")
            .query(&[("tagid", tag), ("pn", pn)])
            .send()
            .await
            .unwrap();
        let mids = resp.json::<Response>().await.unwrap().data;
        for FollowingDetail { mid } in mids {
            let resp = client
                .post("https://api.bilibili.com/x/relation/modify")
                .form(&[
                    ("fid", &*mid.to_string()),
                    ("act", "2"),
                    ("csrf", csrf),
                    ("re_src", "11"),
                ])
                .send()
                .await?;

            let text = resp.text().await?;

            info!("{text}");
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    Ok(())
}
