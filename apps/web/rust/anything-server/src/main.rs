use axum::{
    http::{HeaderValue, Method},
    middleware::{self},
    routing::{delete, get, post, put},
    Router,
};
use dotenv::dotenv;
use postgrest::Postgrest;
use std::env;
use std::sync::Arc;
use tokio::sync::{watch, Semaphore};
use tower_http::cors::CorsLayer;

mod api;
mod auth;
mod bundler;
mod execution_planner;
mod marketplace;
mod secrets;
mod task_engine;
mod task_types;
mod trigger_engine;
mod workflow_types;

#[macro_use]
extern crate slugify;

pub struct AppState {
    anything_client: Arc<Postgrest>,
    marketplace_client: Arc<Postgrest>,
    semaphore: Arc<Semaphore>,
    task_signal: watch::Sender<()>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let supabase_url = env::var("SUPABASE_URL").expect("SUPABASE_URL must be set");
    let supabase_api_key = env::var("SUPABASE_API_KEY").expect("SUPABASE_API_KEY must be set");

    //Anything Schema for Application
    let anything_client = Arc::new(
        Postgrest::new(supabase_url.clone())
            .schema("anything")
            .insert_header("apikey", supabase_api_key.clone()),
    );

    //Marketplace Schema for Managing Templates etc
    let marketplace_client = Arc::new(
        Postgrest::new(supabase_url.clone())
            .schema("marketplace")
            .insert_header("apikey", supabase_api_key.clone()),
    );

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::DELETE,
            Method::PUT,
            Method::OPTIONS,
        ])
        .allow_headers([hyper::header::AUTHORIZATION, hyper::header::CONTENT_TYPE]);

    let (task_signal, _) = watch::channel(());

    let state = Arc::new(AppState {
        anything_client: anything_client.clone(),
        marketplace_client: marketplace_client.clone(),
        semaphore: Arc::new(Semaphore::new(5)),
        task_signal,
    });

    let app = Router::new()
        .route("/", get(api::root))
        .route("/workflows", get(api::get_workflows))
        .route("/workflow/:id", get(api::get_workflow))
        .route("/workflow/:id/versions", get(api::get_flow_versions))
        .route(
            "/workflow/:workflow_id/version/:workflow_version_id",
            put(api::update_workflow_version),
        )
        .route(
            "/workflow/:workflow_id/version/:workflow_version_id/publish",
            put(api::publish_workflow_version),
        )
        .route("/workflow", post(api::create_workflow))
        .route("/workflow/:id", delete(api::delete_workflow))
        .route("/workflow/:id", put(api::update_workflow))
        .route("/actions", get(api::get_actions))
        //Marketplace
        .route(
            "/marketplace/:workflow_id/publish",
            post(marketplace::publish_workflow_to_marketplace),
        )
        //Tasks
        .route("/tasks", get(api::get_tasks))
        .route("/tasks/:workflow_id", get(api::get_task_by_workflow_id))
        //Charts
        .route(
            "/charts/:workflow_id/tasks/:start_date/:end_date/:time_unit",
            get(api::get_task_status_counts_by_workflow_id),
        )
        // Secrets
        .route("/secrets", get(secrets::get_decrypted_secrets))
        .route("/secret", post(secrets::create_secret))
        .route("/secret", put(secrets::update_secret))
        .route("/secret/:id", delete(secrets::delete_secret))
        // Users Testing Workflows
        //Test Workflows
        .route(
            "/testing/workflow/:workflow_id/version/:workflow_version_id",
            get(api::test_workflow),
        )
        .route(
            "/testing/workflow/:workflow_id/version/:workflow_version_id/session/:session_id",
            get(api::get_test_session_results),
        )
        //Test Actions
        .route(
            "/testing/workflow/:workflow_id/version/:workflow_version_id/action/:action_id",
            get(api::test_action),
        )
        .layer(middleware::from_fn(auth::middleware))
        .layer(cors)
        .with_state(state.clone());

    // let url = Wasm::url("https://github.com/extism/plugins/releases/latest/download/count_vowels.wasm");
    // let manifest = Manifest::new([url]);
    // let plugin = Arc::new(Mutex::new(
    //     Plugin::new(&manifest, [], true).unwrap()
    // ));

    // Create a semaphore to limit the number of concurrent tasks
    // let semaphore = Arc::new(Semaphore::new(5));

    // Spawn task processing loop
    // Keeps making progress on work that is meant to be down now.
    tokio::spawn(task_engine::task_processing_loop(state.clone()));

    // // Spawn cron job loop
    // // Initiates work to be done on schedule tasks
    // tokio::spawn(trigger_engine::cron_job_loop(state.clone()));

    // Run the API server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
