use {{RUST_CRATE}}::app;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    println!("API: http://localhost:{port}");
    axum::serve(listener, app())
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

async fn shutdown() {
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("handler SIGTERM disponível em WSL/Linux");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = terminate.recv() => {},
    }
}
