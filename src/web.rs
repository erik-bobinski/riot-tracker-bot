use tower_http::services::ServeDir;

pub async fn serve(port: u16) {
    let app = axum::Router::new().fallback_service(ServeDir::new("static"));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("failed to bind web server port");

    axum::serve(listener, app)
        .await
        .expect("web server crashed");
}
