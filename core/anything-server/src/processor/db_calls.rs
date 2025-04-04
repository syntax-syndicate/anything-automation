use chrono::Utc;
use dotenv::dotenv;
use serde_json::Value;
use std::collections::HashSet;
use std::{env, sync::Arc};
use tracing::debug;
use uuid::Uuid;

use crate::system_plugins::http::http_plugin::parse_headers;
use crate::types::{
    task_types::{FlowSessionStatus, Task, TaskStatus, TriggerSessionStatus},
    workflow_types::DatabaseFlowVersion,
};
use crate::AppState;
use chrono::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateFlowSesssionInput {
    pub flow_session_status: String,
    pub trigger_session_status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateTaskInput {
    pub task_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

pub async fn get_workflow_definition(
    state: Arc<AppState>,
    workflow_id: &Uuid,
    version_id: Option<&Uuid>, // Make version_id optional since webhooks don't have it
) -> Result<DatabaseFlowVersion, String> {
    println!(
        "[PROCESSOR DB CALLS] Getting workflow definition for workflow_id: {}, version_id: {:?}",
        workflow_id, version_id
    );
    //Super User Access
    dotenv().ok();
    let supabase_service_role_api_key = env::var("SUPABASE_SERVICE_ROLE_API_KEY")
        .expect("SUPABASE_SERVICE_ROLE_API_KEY must be set");

    // Get flow version from database
    let mut query = state
        .anything_client
        .from("flow_versions")
        .eq("flow_id", workflow_id.to_string());

    // If version_id is provided, use it. Otherwise get published version
    if let Some(version) = version_id {
        query = query.eq("flow_version_id", version.to_string());
    } else {
        query = query.eq("published", "true");
    }

    let response = query
        .auth(&supabase_service_role_api_key)
        .select("*")
        .single()
        .execute()
        .await
        .map_err(|e| {
            println!(
                "[PROCESSOR DB CALLS] Failed to execute workflow definition request: {}",
                e
            );
            format!("Failed to execute request: {}", e)
        })?;

    let response_body = response.text().await.map_err(|e| {
        println!(
            "[PROCESSOR DB CALLS] Failed to read workflow definition response: {}",
            e
        );
        format!("Failed to read response body: {}", e)
    })?;

    let workflow_version: DatabaseFlowVersion =
        serde_json::from_str(&response_body).map_err(|e| {
            println!("[PROCESSOR DB CALLS] No workflow version found: {}", e);
            String::from("No workflow version found")
        })?;

    println!("[PROCESSOR DB CALLS] Successfully retrieved workflow definition");
    Ok(workflow_version)
}

pub async fn get_session_tasks(
    state: Arc<AppState>,
    flow_session_id: &Uuid, //UUID
) -> Result<Vec<Task>, String> {
    println!(
        "[PROCESSOR DB CALLS] Fetching tasks for flow_session_id {}",
        flow_session_id
    );

    dotenv().ok();
    let supabase_service_role_api_key = env::var("SUPABASE_SERVICE_ROLE_API_KEY")
        .expect("SUPABASE_SERVICE_ROLE_API_KEY must be set");

    let response = state
        .anything_client
        .from("tasks")
        .auth(supabase_service_role_api_key)
        .select("*")
        .eq("flow_session_id", flow_session_id.to_string())
        .order("processing_order.asc")
        .execute()
        .await
        .map_err(|e| {
            println!(
                "[PROCESSOR DB CALLS] Failed to execute session tasks request: {}",
                e
            );
            format!("Failed to execute request: {}", e)
        })?;

    let response_body = response.text().await.map_err(|e| {
        println!(
            "[PROCESSOR DB CALLS] Failed to read session tasks response: {}",
            e
        );
        format!("Failed to read response body: {}", e)
    })?;

    let tasks: Vec<Task> = serde_json::from_str(&response_body).map_err(|e| {
        println!("[PROCESSOR DB CALLS] Failed to parse tasks: {}", e);
        format!("Failed to parse tasks: {}", e)
    })?;

    if tasks.is_empty() {
        println!(
            "[PROCESSOR DB CALLS] No tasks found for session {}",
            flow_session_id
        );
        return Err("No tasks found for session".to_string());
    }

    println!(
        "[PROCESSOR DB CALLS] Successfully retrieved {} tasks",
        tasks.len()
    );
    Ok(tasks)
}

pub async fn create_task(state: Arc<AppState>, task: &Task) -> Result<(), String> {
    println!("[PROCESSOR DB CALLS] Creating new task");
    dotenv().ok();
    let supabase_service_role_api_key = env::var("SUPABASE_SERVICE_ROLE_API_KEY")
        .expect("SUPABASE_SERVICE_ROLE_API_KEY must be set");

    let response = state
        .anything_client
        .from("tasks")
        .auth(supabase_service_role_api_key)
        .insert(
            serde_json::to_value(task)
                .map_err(|e| {
                    println!("[PROCESSOR DB CALLS] Failed to serialize task: {}", e);
                    format!("Failed to serialize task: {}", e)
                })?
                .to_string(),
        )
        .execute()
        .await
        .map_err(|e| {
            println!(
                "[PROCESSOR DB CALLS] Failed to execute create task request: {}",
                e
            );
            format!("Failed to execute request: {}", e)
        })?;

    let response_body = response.text().await.map_err(|e| {
        println!(
            "[PROCESSOR DB CALLS] Failed to read create task response: {}",
            e
        );
        format!("Failed to read response body: {}", e)
    })?;

    let tasks: Vec<Task> = serde_json::from_str(&response_body).map_err(|e| {
        println!("[PROCESSOR DB CALLS] Failed to parse created task: {}", e);
        format!("Failed to parse created task: {}", e)
    })?;

    let task = tasks.into_iter().next().ok_or_else(|| {
        println!("[PROCESSOR DB CALLS] No task was created");
        "No task was created".to_string()
    })?;

    println!("[PROCESSOR DB CALLS] Successfully created task");
    Ok(())
}

//Send just the data we need. Safer to not update every key.
pub async fn update_task_status(
    state: Arc<AppState>,
    task_id: &Uuid,
    status: &TaskStatus,
    context: Option<Value>,
    result: Option<Value>,
    error: Option<Value>,
    started_at: Option<DateTime<Utc>>,
    ended_at: Option<DateTime<Utc>>,
) -> Result<(), String> {
    println!(
        "[PROCESSOR DB CALLS] Updating task {} status to {}",
        task_id,
        status.as_str()
    );
    dotenv().ok();
    let supabase_service_role_api_key = env::var("SUPABASE_SERVICE_ROLE_API_KEY")
        .expect("SUPABASE_SERVICE_ROLE_API_KEY must be set");

    //Remove sensitive headers from context
    let cleaned_context = if let Some(context) = context {
        Some(redact_headers_from_context(&context))
    } else {
        None
    };

    let input = UpdateTaskInput {
        task_status: status.as_str().to_string(),
        started_at,
        ended_at,
        result,
        context: cleaned_context,
        error,
    };

    state
        .anything_client
        .from("tasks")
        .auth(supabase_service_role_api_key)
        .eq("task_id", &task_id.to_string())
        .update(serde_json::to_string(&input).map_err(|e| {
            println!(
                "[PROCESSOR DB CALLS] Failed to serialize update input: {}",
                e
            );
            format!("Failed to serialize input: {}", e)
        })?)
        .execute()
        .await
        .map_err(|e| {
            println!(
                "[PROCESSOR DB CALLS] Failed to execute update task request: {}",
                e
            );
            format!("Failed to execute request: {}", e)
        })?;

    println!("[PROCESSOR DB CALLS] Successfully updated task status");
    Ok(())
}

pub async fn update_flow_session_status(
    state: &AppState,
    flow_session_id: &Uuid,
    flow_session_status: &FlowSessionStatus,
    trigger_session_status: &TriggerSessionStatus,
) -> Result<(), String> {
    println!(
        "[PROCESSOR DB CALLS] Updating flow session {} status to {} and trigger status to {}",
        flow_session_id,
        flow_session_status.as_str(),
        trigger_session_status.as_str()
    );
    dotenv().ok();
    let supabase_service_role_api_key = env::var("SUPABASE_SERVICE_ROLE_API_KEY")
        .expect("SUPABASE_SERVICE_ROLE_API_KEY must be set");

    let input = UpdateFlowSesssionInput {
        flow_session_status: flow_session_status.as_str().to_string(),
        trigger_session_status: trigger_session_status.as_str().to_string(),
    };

    state
        .anything_client
        .from("tasks")
        .auth(supabase_service_role_api_key)
        .eq("flow_session_id", &flow_session_id.to_string())
        .update(serde_json::to_string(&input).map_err(|e| {
            println!(
                "[PROCESSOR DB CALLS] Failed to serialize update input: {}",
                e
            );
            format!("Failed to serialize input: {}", e)
        })?)
        .execute()
        .await
        .map_err(|e| {
            println!(
                "[PROCESSOR DB CALLS] Failed to execute update flow session request: {}",
                e
            );
            format!("Failed to execute request: {}", e)
        })?;

    println!("[PROCESSOR DB CALLS] Successfully updated flow session status");
    Ok(())
}

pub fn redact_headers_from_context(context: &Value) -> Value {
    let mut new_context = context.clone();

    // Parse headers using parse_headers helper
    let headers = parse_headers(context);

    // Create redacted headers object
    let redacted_headers = headers
        .into_iter()
        .map(|(key, _value)| {
            (
                key,
                "REDACTED_FROM_VIEWING_HERE_FOR_SECURITY_REASONS_BY_ANYTHING".to_string(),
            )
        })
        .collect::<Vec<_>>();

    // Convert back to Value object
    let headers_obj = redacted_headers
        .into_iter()
        .map(|(k, v)| (k, Value::String(v)))
        .collect();

    // Update the context with redacted headers
    if let Some(headers) = new_context.get_mut("headers") {
        *headers = Value::Object(headers_obj);
    }

    new_context
}
