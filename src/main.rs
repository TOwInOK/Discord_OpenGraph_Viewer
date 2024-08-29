use axum::{
    extract::{ConnectInfo, Path, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::get,
    serve, Router,
};
use data::UserInfo;
use miniserde::json;
use std::{
    collections::HashMap,
    env::var,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::net::TcpListener;
use tokio::time::interval;
use tracing::{error, info, instrument, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug)]
struct AppState {
    token: String,
    cache: Mutex<HashMap<String, (UserInfo, Instant)>>,
}

mod data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing::subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .finish(),
    )
    .ok();

    let state = Arc::new(AppState {
        token: std::env::var("DS_TOKEN").expect("DS_TOKEN must be set"),
        cache: Mutex::new(HashMap::new()),
    });

    {
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(3600));
            loop {
                interval.tick().await;
                let mut cache = state.cache.lock().unwrap();
                cache.clear();
                info!("Cache cleared");
            }
        });
    }

    let ip = format!(
        "{}:{}",
        var("ADDRESS").expect("ADDRESS must be set"),
        var("PORT").expect("PORT must be set")
    );

    let app = Router::new().route("/id/:id", get(card)).with_state(state);

    serve(
        TcpListener::bind(ip)
            .await
            .expect("fail to launch on current port"),
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}

#[instrument(skip(headers, state))]
async fn card(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // contains json attr
    let json: bool = headers
        .get("accept")
        .and_then(|val| val.to_str().ok())
        .map_or(false, |accept| accept.contains("application/json"));
    // Generate card fn
    let generate = |data: &UserInfo, json: bool| match !json {
        true => data.generate_card().into_response(),
        false => json::to_string(data).into_response(),
    };
    // Check cache
    {
        let cache = state.cache.lock().unwrap();
        if let Some((user_info, cache_time)) = cache.get(&id) {
            if Instant::now().duration_since(*cache_time) <= Duration::from_secs(3600) {
                info!("Use cached info");
                return generate(user_info, json);
            }
        }
    }
    info!("Cached id not found");

    // Fetch user
    let user_info = match data::UserInfo::search(&id, &state.token).await {
        Ok(info) => info,
        Err(e) => {
            error!("No one user found: {}", e);
            return "Error fetching user info".into_response();
        }
    };

    // Update cache
    {
        info!("Cache result");
        let mut cache = state.cache.lock().unwrap();
        cache.insert(id.clone(), (user_info.clone(), Instant::now()));
        info!("sucsecsfully cached")
    }

    generate(&user_info, json)
}
