use serde::Deserialize;

use crate::Client;
use crate::Result;

#[derive(Deserialize)]
struct Response {
    code: i32,
    message: String,
    ttl: i32,
    data: UserInfo,
}

/// some fileds are ignored
#[derive(Deserialize)]
pub struct UserInfo {
    pub mid: u64,
    pub uname: String,
}

pub async fn my_info(client: &mut Client) -> Result<UserInfo> {
    Ok(client
        .get("https://api.bilibili.com/x/web-interface/nav")
        .send()
        .await?
        .json::<Response>()
        .await?
        .data)
}
