pub fn setup() -> eyre::Result<()> {
    color_eyre::install()?;

    tracing_subscriber::fmt::init();

    Ok(())
}
