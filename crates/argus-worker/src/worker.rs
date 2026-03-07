use anyhow::Result;

pub async fn run() -> Result<()> {
    tracing::info!("argus worker started");
    Ok(())
}
