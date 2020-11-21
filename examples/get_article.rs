use std::time::Duration;

use brokaw::types::command as cmd;
use brokaw::{ClientConfig, ConnectionConfig};

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let mut client = ClientConfig::default()
        .connection_config(
            ConnectionConfig::default()
                .read_timeout(Some(Duration::from_secs(10)))
                .to_owned(),
        )
        .group(Some("mozilla.dev.platform"))
        .connect(("news.mozilla.org", 119))
        .await?;

    let highest_article = client.group().unwrap().high;

    let article = client
        .article(cmd::Article::Number(highest_article))
        .await
        .and_then(|a| a.to_text())?;

    println!("~~~ 📰 `{}` ~~~", article.message_id());
    println!("~~~ Headers ~~~");
    article.headers().iter().for_each(|header| {
        println!("Header {} --> {:?}", header.name, header.content);
    });

    println!("~~~ Body ~~~");
    article.body().iter().for_each(|line| println!("{}", line));
    println!("~~~ 👋🏾 ~~~");

    Ok(())
}
