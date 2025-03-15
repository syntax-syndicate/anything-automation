use aws_config::default_provider::credentials::DefaultCredentialsChain;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use std::sync::Arc;
use uuid::Uuid;

use crate::{supabase_jwt_middleware::User, AppState};

use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileAccessType {
    Private, // Requires signed URL
    Public,  // Always accessible via CDN
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    file_id: String,
    file_name: String,
    file_size: i64,
    content_type: String,
    // created_at: chrono::DateTime<chrono::Utc>,
    account_id: String,
    path: Option<String>,        // For future folder support
    public_url: Option<String>,  // For public files
    access_type: FileAccessType, // Controls how the file can be accessed
}

#[derive(Debug, Serialize)]
pub struct FileResponse {
    file_id: String,
    file_name: String,
    content_type: String,
    size: i64,
    url: String,
    base64: Option<String>,
}

// Initialize R2 client
async fn get_r2_client() -> S3Client {
    println!("[FILES] Initializing R2 client");
    let r2_account_id = std::env::var("R2_ACCOUNT_ID").expect("R2_ACCOUNT_ID must be set");
    let r2_access_key_id = std::env::var("R2_ACCESS_KEY_ID").expect("R2_ACCESS_KEY_ID must be set");
    let r2_secret_access_key =
        std::env::var("R2_SECRET_ACCESS_KEY").expect("R2_SECRET_ACCESS_KEY must be set");

    let credentials = Credentials::new(
        r2_access_key_id,
        r2_secret_access_key,
        None,
        None,
        "anything-r2",
    );

    let config = aws_sdk_s3::Config::builder()
        .behavior_version_latest()
        .region(Region::new("auto"))
        .endpoint_url(format!(
            "https://{}.r2.cloudflarestorage.com",
            r2_account_id
        ))
        .credentials_provider(credentials)
        .build();

    println!("[FILES] R2 client initialized successfully");
    S3Client::from_conf(config)
}

// Get files for an account
pub async fn get_files(
    Path(account_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    println!("[FILES] Getting files for account: {}", account_id);
    let client = &state.anything_client;

    let response = match client
        .from("files")
        .auth(&user.jwt)
        .eq("account_id", &account_id)
        .select("*")
        .order("created_at.desc")
        .execute()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            println!("[FILES] Failed to fetch files: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch files").into_response();
        }
    };

    let body = match response.text().await {
        Ok(body) => body,
        Err(e) => {
            println!("[FILES] Failed to read response: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read response").into_response();
        }
    };

    let files: Value = match serde_json::from_str(&body) {
        Ok(files) => files,
        Err(e) => {
            println!("[FILES] Failed to parse files: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse files").into_response();
        }
    };

    println!(
        "[FILES] Successfully retrieved {} files",
        files.as_array().map_or(0, |a| a.len())
    );
    Json(files).into_response()
}

// Upload a file
pub async fn upload_file(
    Path((account_id, access)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    println!("[FILES] Starting file upload for account: {}", account_id);
    let r2_client = get_r2_client().await;
    let bucket = std::env::var("R2_BUCKET").expect("R2_BUCKET must be set");
    let cdn_domain = std::env::var("R2_PUBLIC_DOMAIN").expect("R2_PUBLIC_DOMAIN must be set");

    // Check access type from path parameter
    let is_private = access == "private";
    println!(
        "[FILES] File access type: {}",
        if is_private { "private" } else { "public" }
    );

    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap_or("unnamed").to_string();
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let data = field.bytes().await.unwrap();

        println!(
            "[FILES] Processing file: {} ({} bytes)",
            file_name,
            data.len()
        );

        let file_id = Uuid::new_v4().to_string();
        // Simplified key structure - no folders
        let r2_key = format!("{}/{}", account_id, file_name);

        let mut put_object = r2_client
            .put_object()
            .bucket(&bucket)
            .key(&r2_key)
            .body(ByteStream::from(data.clone()))
            .content_type(content_type.clone());

        // Set ACL based on access type
        if !is_private {
            put_object = put_object.acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead);
        }

        match put_object.send().await {
            Ok(_) => {
                println!("[FILES] Successfully uploaded file to R2: {}", r2_key);
                let file_metadata = FileMetadata {
                    file_id: file_id.clone(),
                    file_name,
                    file_size: data.len() as i64,
                    content_type,
                    // created_at: chrono::Utc::now(),
                    account_id: account_id.clone(),
                    path: None,
                    public_url: if !is_private {
                        Some(format!("{}/{}", cdn_domain, r2_key))
                    } else {
                        None
                    },
                    access_type: if is_private {
                        FileAccessType::Private
                    } else {
                        FileAccessType::Public
                    },
                };

                let client = &state.anything_client;
                let response = match client
                    .from("files")
                    .auth(&user.jwt)
                    .insert(serde_json::to_string(&file_metadata).unwrap())
                    .execute()
                    .await
                {
                    Ok(response) => response,
                    Err(e) => {
                        println!("[FILES] Failed to store file metadata: {:?}", e);
                        // Cleanup R2 if database insert fails
                        let _ = r2_client
                            .delete_object()
                            .bucket(&bucket)
                            .key(&r2_key)
                            .send()
                            .await;
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to store file metadata",
                        )
                            .into_response();
                    }
                };

                // Check the actual response status
                if response.status().is_success() {
                    println!("[FILES] Successfully stored file metadata for: {}", file_id);
                    return Json(json!({
                        "status": "success",
                        "file_id": file_id
                    }))
                    .into_response();
                } else {
                    println!(
                        "[FILES] Failed to store file metadata. Status: {}, Body: {:?}",
                        response.status(),
                        response.text().await.unwrap_or_default()
                    );
                    // Cleanup R2 if database insert fails
                    let _ = r2_client
                        .delete_object()
                        .bucket(&bucket)
                        .key(&r2_key)
                        .send()
                        .await;
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to store file metadata",
                    )
                        .into_response();
                }
            }
            Err(e) => {
                println!("[FILES] Failed to upload file to R2: {:?}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to upload file to storage",
                )
                    .into_response();
            }
        }
    }

    println!("[FILES] No file provided in request");
    (StatusCode::BAD_REQUEST, "No file provided").into_response()
}

// Delete a file
pub async fn delete_file(
    Path((account_id, file_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    println!(
        "[FILES] Deleting file {} for account {}",
        file_id, account_id
    );
    let client = &state.anything_client;
    let r2_client = get_r2_client().await;
    let bucket = std::env::var("R2_BUCKET").expect("R2_BUCKET must be set");

    // First, get the file metadata
    let response = match client
        .from("files")
        .auth(&user.jwt)
        .eq("file_id", &file_id)
        .eq("account_id", &account_id)
        .select("*")
        .single()
        .execute()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            println!("[FILES] File not found: {:?}", e);
            return (StatusCode::NOT_FOUND, "File not found").into_response();
        }
    };

    let file_metadata: FileMetadata = match response.json().await {
        Ok(metadata) => metadata,
        Err(e) => {
            println!("[FILES] Failed to parse file metadata: {:?}", e);
            return (StatusCode::NOT_FOUND, "File not found").into_response();
        }
    };

    // Delete from R2
    let r2_key = format!("{}_{}", file_id, file_metadata.file_name);
    println!("[FILES] Deleting file from R2: {}", r2_key);

    match r2_client
        .delete_object()
        .bucket(&bucket)
        .key(&r2_key)
        .send()
        .await
    {
        Ok(_) => {
            println!("[FILES] Successfully deleted file from R2");
            // Delete metadata from database
            match client
                .from("files")
                .auth(&user.jwt)
                .eq("file_id", &file_id)
                .eq("account_id", &account_id)
                .delete()
                .execute()
                .await
            {
                Ok(_) => {
                    println!("[FILES] Successfully deleted file metadata");
                    Json(json!({"status": "success"})).into_response()
                }
                Err(e) => {
                    println!("[FILES] Failed to delete file metadata: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to delete file metadata",
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            println!("[FILES] Failed to delete file from R2: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete file from storage",
            )
                .into_response()
        }
    }
}

// Get a pre-signed download URL for a file
pub async fn get_file_download_url(
    Path((account_id, file_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    println!(
        "[FILES] Getting download URL for file {} in account {}",
        file_id, account_id
    );
    let client = &state.anything_client;
    let r2_client = get_r2_client().await;
    let bucket = std::env::var("R2_BUCKET").expect("R2_BUCKET must be set");

    // Get file metadata
    let response = match client
        .from("files")
        .auth(&user.jwt)
        .eq("file_id", &file_id)
        .eq("account_id", &account_id)
        .select("*")
        .single()
        .execute()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            println!("[FILES] File not found: {:?}", e);
            return (StatusCode::NOT_FOUND, "File not found").into_response();
        }
    };

    let file_metadata: FileMetadata = match response.json().await {
        Ok(metadata) => metadata,
        Err(e) => {
            println!("[FILES] Failed to parse file metadata: {:?}", e);
            return (StatusCode::NOT_FOUND, "File not found").into_response();
        }
    };

    // Simplified key structure
    let r2_key = format!("{}_{}", file_id, file_metadata.file_name);

    // If public, return CDN URL
    if let Some(public_url) = file_metadata.public_url {
        println!("[FILES] Returning public CDN URL for file");
        return Json(json!({
            "download_url": public_url
        }))
        .into_response();
    }

    println!("[FILES] Generating presigned URL for private file");
    // If private, generate presigned URL
    let presigned_request = match r2_client
        .get_object()
        .bucket(&bucket)
        .key(&r2_key)
        .presigned(
            aws_sdk_s3::presigning::PresigningConfig::expires_in(std::time::Duration::from_secs(
                3600,
            ))
            .unwrap(),
        )
        .await
    {
        Ok(url) => url,
        Err(e) => {
            println!("[FILES] Failed to generate presigned URL: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate download URL",
            )
                .into_response();
        }
    };

    println!("[FILES] Successfully generated download URL");
    Json(json!({
        "download_url": presigned_request.uri().to_string()
    }))
    .into_response()
}
