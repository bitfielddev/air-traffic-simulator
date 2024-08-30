use std::{fs::File, io::Cursor, path::PathBuf, process::Command, sync::Arc};

use axum::routing::get;
use engine::{
    engine::Engine,
    util::{AirportCode, AirportStateId, PlaneStateId},
};
use eyre::Result;
use socketioxide::{
    extract::{AckSender, Data, SocketRef, State},
    SocketIo,
};
use tokio::{
    net::TcpListener,
    sync::RwLock,
    time::{Duration, Instant},
};
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};
use tracing::{debug, error, info, warn};

#[tracing::instrument(skip_all)]
fn build_client(client_config: Option<&str>) -> Result<tempfile::TempDir> {
    let zip_file = include_bytes!(concat!(env!("OUT_DIR"), "/client.tar.gz"));
    let dir = tempfile::tempdir()?;

    info!(to=?dir.path(), "Extracting to temporary folder");
    std::fs::write(dir.path().join("client.tar.gz"), zip_file)?;
    let output = Command::new("tar")
        .args([
            "xf",
            &dir.path().join("client.tar.gz").to_string_lossy(),
            "--directory",
            &dir.path().to_string_lossy(),
        ])
        .output()?;
    warn!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    warn!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    info!(to=?dir.path(), "Building client");
    if let Some(client_config) = client_config {
        std::fs::write(
            dir.path().join("src").join("config").join("config.js"),
            client_config,
        )?;
    }
    let output = Command::new("npm")
        .args(["run", "build"])
        .current_dir(dir.path())
        .output()?;
    warn!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    warn!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    Ok(dir)
}

#[expect(clippy::needless_pass_by_value)]
fn websocket_connect(socket: SocketRef) {
    info!("Socket.IO connected: {:?} {:?}", socket.ns(), socket.id);

    socket.on(
        "plane",
        |ack: AckSender, Data(uuid): Data<PlaneStateId>, engine_arc: State<Arc<RwLock<Engine>>>| async move {
            let engine = engine_arc.read().await;
            let _ = ack
                .send(engine.state.plane(&uuid))
                .inspect_err(|e| error!(ev="plane", "{e:#}"));
        },
    );

    socket.on(
        "airport",
        |ack: AckSender, Data(id): Data<AirportStateId>, engine_arc: State<Arc<RwLock<Engine>>>| async move {
            let engine = engine_arc.read().await;
            let _ = ack
                .send(engine.state.airport(&id))
                .inspect_err(|e| error!(ev="airport", "{e:#}"));
        },
    );

    socket.on(
        "airport_departures",
        |ack: AckSender, Data(code): Data<AirportCode>, engine_arc: State<Arc<RwLock<Engine>>>| async move {
            let engine = engine_arc.read().await;
            let _ = ack
                .send([engine.state.airport_departures(&code).map(|a| a.id).collect::<Vec<_>>()])
                .inspect_err(|e| error!(ev="airport_departures", "{e:#}"));
        },
    );

    socket.on(
        "airport_arrivals",
        |ack: AckSender, Data(code): Data<AirportCode>, engine_arc: State<Arc<RwLock<Engine>>>| async move {
            let engine = engine_arc.read().await;
            let _ = ack
                .send([engine.state.airport_arrivals(&code).map(|a| a.id).collect::<Vec<_>>()])
                .inspect_err(|e| error!(ev="airport_arrivals", "{e:#}"));
        },
    );

    socket.on(
        "world_data",
        |ack: AckSender, engine_arc: State<Arc<RwLock<Engine>>>| async move {
            let engine = engine_arc.read().await;
            let _ = ack
                .send(&engine.world)
                .inspect_err(|e| error!(ev = "world_data", "{e:#}"));
        },
    );

    socket.on(
        "engine_config",
        |ack: AckSender, engine_arc: State<Arc<RwLock<Engine>>>| async move {
            let engine = engine_arc.read().await;
            let _ = ack
                .send(&engine.config)
                .inspect_err(|e| error!(ev = "engine_config", "{e:#}"));
        },
    );
}

#[tracing::instrument(skip_all)]
pub async fn server(engine: Engine, client_config: Option<&str>) -> Result<()> {
    let engine_arc = Arc::new(RwLock::new(engine));
    let (layer, io) = SocketIo::builder()
        .with_state(Arc::clone(&engine_arc))
        .build_layer();
    io.ns("/", websocket_connect);

    let client = build_client(client_config)?;
    let client_dist = client.path().join("dist");

    let app = axum::Router::new()
        .nest_service(
            "/",
            ServeDir::new(&client_dist)
                .not_found_service(ServeFile::new(client_dist.join("index.html"))),
        )
        .layer(layer)
        .layer(CorsLayer::permissive());

    tokio::spawn(async move {
        #[expect(clippy::infinite_loop)]
        loop {
            let start = Instant::now();
            let mut engine = engine_arc.write().await;
            let (removed, state) = engine.tick();
            drop(engine);
            let _ = io
                .bin([state])
                .emit("state", [removed])
                .inspect_err(|e| error!(ev = "state", "{e:#}"));
            info!(delta=?start.elapsed());
            tokio::time::sleep(Duration::from_secs(1) - start.elapsed()).await;
        }
    });

    info!("Starting server");
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
