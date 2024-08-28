use std::io::Read;

use axum::routing::get;
use engine::{config::Config, engine::Engine, world_data::WorldData};
use eyre::Result;
use itertools::Itertools;
use serde::de::DeserializeOwned;
use socketioxide::{
    extract::{Data, SocketRef},
    SocketIo,
};
use tokio::{
    net::TcpListener,
    time::{Duration, Instant},
};
use tracing::{error, info};

#[expect(clippy::needless_pass_by_value)]
fn websocket_connect(socket: SocketRef, Data(data): Data<()>) {
    info!("Socket.IO connected: {:?} {:?}", socket.ns(), socket.id);
    socket.emit("welcome", data).ok();
}

#[tracing::instrument]
pub async fn server(engine: Engine) -> Result<()> {
    let (layer, io) = SocketIo::new_layer();
    io.ns("/ws", websocket_connect);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(layer);

    info!("Starting server");

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    tokio::spawn(async move {
        #[expect(clippy::infinite_loop)]
        loop {
            let start = Instant::now();
            engine.tick();
            let _ = io
                .emit("state", &engine.state)
                .inspect_err(|e| error!("{e:#}"));
            info!(delta=?start.elapsed());
            tokio::time::sleep(Duration::from_secs(1) - start.elapsed()).await;
        }
    });

    Ok(())
}
