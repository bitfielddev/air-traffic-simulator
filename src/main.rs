use std::io::Read;

use axum::routing::get;
use clap::Parser;
use clio::Input;
use color_eyre::{
    eyre::{eyre, WrapErr},
    Report, Result,
};
use engine::engine::Engine;
use itertools::Itertools;
use serde::de::DeserializeOwned;
use socketioxide::SocketIo;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(long, short, value_parser, required = true)]
    world_data: Input,

    #[arg(long, short, value_parser, required = true)]
    config: Input,
}

fn parse<T: DeserializeOwned>(mut input: Input) -> Result<T> {
    let mut errors = Vec::<Report>::new();

    match rmp_serde::from_read(input.clone()) {
        Ok(v) => return Ok(v),
        Err(e) => errors.push(e.into()),
    }

    let mut buf = String::new();
    input.read_to_string(&mut buf)?;

    match serde_json::from_str::<serde_json::Value>(&buf) {
        Ok(v) => return serde_json::from_value(v).map_err(Report::from),
        Err(e) => errors.push(e.into()),
    }
    match serde_yaml::from_str::<serde_yaml::Value>(&buf) {
        Ok(v) => return serde_yaml::from_value(v).map_err(Report::from),
        Err(e) => errors.push(e.into()),
    }
    match toml::from_str::<toml::Value>(&buf) {
        Ok(_) => return toml::from_str(&buf).map_err(Report::from),
        Err(e) => errors.push(e.into()),
    }

    Err(eyre!(
        "Unrecognised error\n{}",
        errors.iter().map(|a| format!("{a:#}")).join("\n\n")
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(EnvFilter::from_env("RUST_LOG"))
        .with(fmt::layer())
        .try_init()?;

    let cli = Cli::parse();
    let world_data = parse(cli.world_data).wrap_err("World data file invalid")?;
    let config = parse(cli.config).wrap_err("Config file invalid")?;
    tokio::spawn(async move {
        let mut engine = Engine::new(world_data, config);
        loop {
            let start = tokio::time::Instant::now();
            engine.tick();
            info!(delta=?start.elapsed());
            tokio::time::sleep(tokio::time::Duration::from_secs(1) - start.elapsed()).await;
        }
    });
    let (layer, io) = SocketIo::new_layer();

    // io.ns("/", on_connect);
    // io.ns("/custom", on_connect);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(layer);

    info!("Starting server");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
