use axum::{extract::Multipart, response::IntoResponse, routing::post, Router, extract::{DefaultBodyLimit}};
use axum::http::StatusCode;
use axum::http::{header::{ACCEPT, CONTENT_TYPE}, HeaderValue, Method};

use std::{fs, path::Path};
use tokio::io::AsyncWriteExt;
use tower_http::cors::{CorsLayer};
use sanitize_filename::sanitize;

const PORT: u16 = 8080;

/// Handles file uploads
async fn upload_file(
    mut multipart: Multipart, // No manual limit; DefaultBodyLimit will handle it
) -> impl IntoResponse {
    let upload_dir = "./uploads";

    // Ensure the upload directory exists
    if !Path::new(upload_dir).exists() {
        if let Err(e) = fs::create_dir_all(upload_dir) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create upload directory: {}", e),
            );
        }
    }

    // Process each field in the multipart request
    while let Ok(Some(mut field)) = multipart.next_field().await {
        if let Some(filename) = field.file_name() {
            let safe_filename = sanitize(filename);
            let filepath = format!("{}/{}", upload_dir, safe_filename);

            let mut file = match tokio::fs::File::create(&filepath).await {
                Ok(file) => file,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to create file: {}", e),
                    );
                }
            };

            while let Some(chunk) = field.chunk().await.unwrap_or(None) {
                if let Err(e) = file.write_all(&chunk).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to write file: {}", e),
                    );
                }
            }
        }
    }
    (StatusCode::OK, "File uploaded successfully".to_string())
}

#[tokio::main]
async fn main() {
    // CORS layer to allow requests from any origin
    let cors = CorsLayer::new()
        .allow_methods([Method::POST])
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        // .allow_origin(Any) // Allows all origins; restrict as needed
        .allow_headers([ACCEPT, CONTENT_TYPE]);

    // Build the Axum app
    let app = Router::new()
        .route("/upload", post(upload_file))
        .layer(cors.clone())
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)); // Limit to 10 MB

    println!(
        "{}",
        format!("🚀 Server is running on http://localhost:{}", PORT)
    );

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", PORT))
        .await.unwrap();

    axum::serve(listener, app).await.unwrap();

}
