mod llm;

use rusqlite::Connection;
use reqwest::Client;

use llm::Translator as LlmTramslator;

pub async fn run(db: &mut Connection, series: &[&(u16, String)]) -> anyhow::Result<()> {
    let cli = Client::new();

    let tl = LlmTramslator::new()?;
    tl.translate(cli, db, series).await?;

    Ok(())
}
