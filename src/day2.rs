#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    crate::setup()?;
    Ok(())
}
