use color_eyre::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(EnvFilter::from_env("RUST_LOG"))
        .with(fmt::layer())
        .try_init()?;

    let world_data = serde_yaml::from_str(include_str!("wd.yml"))?;
    let engine_config = serde_yaml::from_str(include_str!("engine_config.yml"))?;
    let engine = air_traffic_simulator::Engine::new(world_data, engine_config);

    air_traffic_simulator::run_server(engine, None).await?;

    Ok(())
}
