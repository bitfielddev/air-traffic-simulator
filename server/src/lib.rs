use std::sync::Arc;

use axum::{body::Bytes, routing::get};
use engine::{
    engine::Engine,
    util::{AirportStateId, PlaneStateId},
};
use eyre::{Report, Result};
use socketioxide::{
    extract::{AckSender, Data, SocketRef, State},
    SocketIo,
};
use tokio::{
    net::TcpListener,
    sync::RwLock,
    time::{Duration, Instant},
};
use tracing::{error, info};

#[expect(clippy::needless_pass_by_value)]
fn websocket_connect(socket: SocketRef, Data(_data): Data<()>) {
    info!("Socket.IO connected: {:?} {:?}", socket.ns(), socket.id);

    socket.on(
        "plane",
        |ack: AckSender, Data(uuid): Data<PlaneStateId>, engine_arc: State<Arc<RwLock<Engine>>>| {
            let engine = engine_arc.blocking_read();
            let _ = ack
                .send(engine.state.plane(&uuid))
                .inspect_err(|e| error!("{e:#}"));
        },
    );

    socket.on(
        "airport",
        |ack: AckSender, Data(id): Data<AirportStateId>, engine_arc: State<Arc<RwLock<Engine>>>| {
            let engine = engine_arc.blocking_read();
            let _ = ack
                .send(engine.state.airport(&id))
                .inspect_err(|e| error!("{e:#}"));
        },
    );

    socket.on(
        "world_data",
        |ack: AckSender, engine_arc: State<Arc<RwLock<Engine>>>| {
            let engine = engine_arc.blocking_read();
            let _ = ack.send(&engine.world).inspect_err(|e| error!("{e:#}"));
        },
    );

    socket.on(
        "config",
        |ack: AckSender, engine_arc: State<Arc<RwLock<Engine>>>| {
            let engine = engine_arc.blocking_read();
            let _ = ack.send(&engine.config).inspect_err(|e| error!("{e:#}"));
        },
    );
}

#[tracing::instrument(skip_all)]
pub async fn server(engine: Engine) -> Result<()> {
    let engine_arc = Arc::new(RwLock::new(engine));
    let (layer, io) = SocketIo::builder()
        .with_state(Arc::clone(&engine_arc))
        .build_layer();
    io.ns("/", websocket_connect);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(layer);

    tokio::spawn(async move {
        #[expect(clippy::infinite_loop)]
        loop {
            let start = Instant::now();
            let mut engine = engine_arc.write().await;
            engine.tick();
            let state = engine.state.coord_state();
            drop(engine);
            let _ = io
                .bin(vec![state])
                .emit("state", ())
                .inspect_err(|e| error!("{e:#}"));
            info!(delta=?start.elapsed());
            tokio::time::sleep(Duration::from_secs(1) - start.elapsed()).await;
        }
    });

    info!("Starting server");
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
