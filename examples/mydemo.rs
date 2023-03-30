use std::{collections::HashMap, time::Duration};

use log::debug;
use pocketbase_sdk_rust::{client::PocketBase, error::Result, user::UserTypes};

/// cargo run --example mydemo
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut pb = PocketBase::new("http://167.71.201.1/")?;

    let mut new_user = HashMap::new();
    new_user.insert(
        "email".to_string(),
        "nguyenvietTrungDemo@gmail.com".to_string(),
    );
    new_user.insert("password".to_string(), "Matkhaucuatoi".to_string());
    new_user.insert("passwordConfirm".to_string(), "Matkhaucuatoi".to_string());
    let res = pb
        .create::<_, _, serde_json::Value>("users", &new_user)
        .await?;
    debug!("{:#?}", res);

    pb.auth_via_email(
        "nguyenvietTrungDemo@gmail.com",
        "Matkhaucuatoi",
        UserTypes::User,
    )
    .await?;

    debug!("Is Auth Store Valid: {}", pb.is_auth_store_valid());

    pb.refresh_token().await?;

    pb.subscribe("users", "*", |event| async move {
        debug!("users Got event {:#?}", event);
    })
    .await?;

    sleep_with_log(Duration::from_secs(60 * 5)).await;
    debug!("After 5m");

    sleep_with_log(Duration::from_secs(60 * 5)).await;
    debug!("Bye!!");
    Ok(())
}

async fn sleep_with_log(duration: Duration) {
    let now = std::time::Instant::now();
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        let pass_time = std::time::Instant::now() - now;
        if duration > pass_time {
            let timeleft = duration - pass_time;
            debug!("Timeleft: {}", timeleft.as_secs());
        } else {
            break;
        }
    }
}
