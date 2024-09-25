use std::time::Duration;

use serde::Deserialize;
use tracing::info;

use crate::Client;
use crate::Result;

#[derive(Deserialize)]
struct TagDetailResponse {
    code: i32,
    message: String,
    ttl: i32,
    data: Vec<FollowingDetail>,
}

#[derive(Deserialize)]
pub struct FollowingDetail {
    pub mid: u64,
}

#[derive(Deserialize)]
struct TagsResponse {
    code: i32,
    message: String,
    ttl: i32,
    data: Vec<Tag>,
}

#[derive(Deserialize)]
pub struct Tag {
    pub tagid: i64,
    pub name: String,
    pub count: u64,
}

pub async fn list_tags(client: &Client) -> Result<Vec<Tag>> {
    let tags_rep = client
        .get("https://api.bilibili.com/x/relation/tags")
        .send()
        .await?
        .json::<TagsResponse>()
        .await?;
    Ok(tags_rep.data)
}

pub async fn unsubcribe_users_with_tag(client: &Client, tag: i64, csrf: &str) -> Result<()> {
    for pn in 1.. {
        let resp = client
            .get("https://api.bilibili.com/x/relation/tag")
            .query(&[("tagid", tag), ("pn", pn)])
            .send()
            .await?;
        let mids = resp.json::<TagDetailResponse>().await?.data;

        if mids.is_empty() {
            info!("un-subscribe finished for tag {tag}");
            break;
        }

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
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    Ok(())
}
