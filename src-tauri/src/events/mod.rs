use tokio::time::{sleep, Duration};
use tauri::{
    AppHandle, Manager
};
use std::{collections::{HashMap, VecDeque}, fs};
use crate::sql::plugin::{select, DbInstances, DB_STRING, execute, Error};
use serde_json::Value as JsonValue;
use tauri::api::path::document_dir;
use std::io::{Result, Error as IOError, ErrorKind};
use uuid::Uuid;

use crate::notifications::Event; 

extern crate chrono;
use chrono::Utc; 

#[derive(Clone, serde::Serialize)]
 struct Payload {
  message: String,
  name: String
}

pub async fn scheduler(app: &AppHandle){
    loop {
        let app_handle = app.clone(); 
       let win =  app_handle.get_window("main").unwrap();
        let window = app.get_window("main").unwrap();

        tokio::spawn(async move {
            process(&app_handle).await;
        });

       sleep(Duration::from_secs(2)).await; 
    }
}

//TODO: write a function to safely extract JSON from Event. This code makes me ill. Do nesting somwhere else
async fn process(app: &AppHandle) {

    let res = fetch_event(app).await; 
    
    let sql_event_id: &str;

    match res {
        Ok(items) => {
            if let Some(item) = items.get(0) {
                if let Some(event_id) = item.get("event_id") {
                     sql_event_id = event_id.as_str().unwrap();
                   
                    if let Some(worker_type) = item.get("worker_type") {
                        
                            if let Some(worker_type_str) = worker_type.as_str() {
                                // Now worker_type_str is a &str
                                match execute_worker_task(app, worker_type_str, item, sql_event_id).await {
                                    Ok(_) => {
                                        mark_as_done(app, sql_event_id.to_string()).await;
                                        println!("event_id: {} marked as COMPLETE after passing through execute_worker_task", event_id);
                                    },
                                    Err(err) => {
                                        println!("Failed to execute worker task: {}", err);
                                    }
                                }
                            } else {
                                // Handle the case where worker_type is not a string
                                println!("Worker type is not a string")
                            }                        
                    } else {
                        println!("event_name not found in the item.");
                    }
                } else { 
                    println!("event_id not found in the item.");
                }
            } else {
                println!("No items in the response.");
            }
        }
        Err(err) => {
            println!("Error: {}", err);
        }
    }
}

//FIXME: we need to switch over to flow_id now that it exists because name changes kill
//when we have an even in the sqlite stack that is not done yet when names change. 
//TODO: also just handle that Error Gracefully when it does happen because it is possible
async fn fetch_event<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> std::result::Result<Vec<HashMap<String, JsonValue>>, Error> {
    // Access the dbInstances from the app's state
    let db_instances = app.state::<DbInstances>(); 
    //make Query
    let db = DB_STRING.to_string();
    let query = "SELECT * FROM events WHERE event_status = $1 ORDER BY created_at ASC LIMIT 1".to_string(); 
    let values = vec![JsonValue::String("PENDING".to_string())];
    
    println!("Fetching Next Event"); 
    // Call the select function with the fetched dbInstances state
    select(db_instances, db, query, values).await
}

async fn create_event<R: tauri::Runtime>(
    app: &AppHandle<R>,
    node: &JsonValue,
    flow_info: &JsonValue,
) -> std::result::Result<(), Error> {
    let db_instances = app.state::<DbInstances>(); 

    let db = DB_STRING.to_string();

    // Extract node details and other required information
    let node_id = node.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let node_type = node.get("type").and_then(|v| v.as_str()).unwrap_or_default();
    let data = node.get("data").and_then(|v| v.as_str()).unwrap_or_default();
    let worker_type = node.get("data")
                          .and_then(|data| data.get("worker_type"))
                          .and_then(|wt| wt.as_str())
                          .unwrap_or_default();

    // Flow specific info (adjust as per your requirement)
    let flow_id = flow_info.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let flow_name = flow_info.get("name").and_then(|v| v.as_str()).unwrap_or_default();
    let flow_version = flow_info.get("version").and_then(|v| v.as_str()).unwrap_or_default();
    // ... (Add other data extraction as needed) ...

    let query = "
        INSERT INTO events (event_id, session_id, node_id, node_type, flow_id, flow_name, flow_version, stage, worker_type, event_status, session_status, created_at, data) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
    ";

    let values = vec![
        JsonValue::String(Uuid::new_v4().to_string()),       // event_id
        JsonValue::String(Uuid::new_v4().to_string()),       // session_id
        JsonValue::String(node_id.to_string()),              // node_id
        JsonValue::String(node_type.to_string()),            // node_type
        JsonValue::String(flow_id.to_string()),              // flow_id
        JsonValue::String(flow_name.to_string()),            // flow_name
        JsonValue::String(flow_version.to_string()),         // flow_version
        JsonValue::String("dev".to_string()),                // stage
        JsonValue::String(worker_type.to_string()),          // worker_type
        JsonValue::String("PENDING".to_string()),            // event_status
        JsonValue::String("PENDING".to_string()),            // session_status
        JsonValue::String(Utc::now().to_rfc3339()),          // created_at
        JsonValue::String(data.to_string()),                 // data
    ];

    match execute(db_instances, db, query.to_string(), values).await {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error adding event to db: {}", e);
            Err(e)          
        },
    }
}


async fn mark_as_done<R: tauri::Runtime>(
    app: &AppHandle<R>,
    event_id: String,
) {
    let db_instances = app.state::<DbInstances>(); 

    let db = DB_STRING.to_string();
    let query = "UPDATE events
    SET event_status = 'COMPLETE'
    WHERE event_id = $1".to_string(); 
    let values = vec![JsonValue::String(event_id)];

    match execute(db_instances, db, query, values).await {
        Ok((affected_rows, last_insert_id)) => {
            println!("Affected rows: {}", affected_rows);
            println!("Last insert ID: {}", last_insert_id);
        }
        Err(e) => {
            println!("Error executing the query: {:?}", e);
        }
    }
}

async fn create_events_from_graph<R: tauri::Runtime>(app: &AppHandle<R>, file_name: &str){

     let toml_document = read_from_documents(file_name).unwrap(); 

      // Convert TOML to serde_json::Value
      let parsed_toml: JsonValue = toml::from_str(&toml_document).expect("Failed to parse TOML");

      println!("{}", parsed_toml); 
      // Convert parsed TOML into JSON Value
      let flow_json_data = serde_json::to_value(parsed_toml).expect("Failed to convert to JSON");

      let work_order = bfs_traversal(&flow_json_data);
      //We now have all the events but including the start event. 

      println!("Found {} pieces of work to build out", work_order.len()); 

      //this loop skips the first item
      for work in work_order.iter().skip(1){
        println!("{}", work); 
   
        if let Some(flow) = flow_json_data.get("flow") {
       let _res =  create_event(app, work, flow).await;
       println!("ID: {} is created as the next item in the work order", work.get("id").unwrap());
       //TODO: give the user the update?
        } else {
            println!("Flow not found in the json_data");
        }
     
      }
}

fn bfs_traversal(json_data: &JsonValue) -> Vec<JsonValue> {
    // Resultant list of work
    let mut work_list = Vec::new();

    // Create a map of node ids to their outgoing edges
    let mut graph = std::collections::HashMap::new();
    if let Some(edges) = json_data.get("edges") {
        for edge in edges.as_array().unwrap() {
            let source = edge.get("source").unwrap().as_str().unwrap();
            let target = edge.get("target").unwrap().as_str().unwrap();

            graph.entry(source.to_string()).or_insert_with(Vec::new).push(target.to_string());
        }
    }

    // Use a BFS queue
    let mut queue = VecDeque::new();

    // Find and enqueue the node with "data.worker_type" = "start"
    if let Some(nodes) = json_data.get("nodes") {
        for node in nodes.as_array().unwrap() {
            if let Some(data) = node.get("data") {
                if data.get("worker_type").map_or(false, |w| w.as_str().unwrap_or("") == "start") {
                    queue.push_back(node.clone());
                    break;  // Since there should be only one start node based on the context
                }
            }
        }
    }

    // BFS traversal
    while !queue.is_empty() {
        let current = queue.pop_front().unwrap();

        // Add current node to the work list
        work_list.push(current.clone());

        // Enqueue neighbors
        if let Some(neighbors) = graph.get(current.get("id").unwrap().as_str().unwrap()) {
            if let Some(nodes) = json_data.get("nodes") {
                for neighbor_id in neighbors {
                    let neighbor = nodes.as_array().unwrap().iter().find(|&node| node.get("id").unwrap().as_str().unwrap() == neighbor_id).unwrap().clone();
                    queue.push_back(neighbor);
                }
            }
        }
    }

    work_list
}

fn read_from_documents(flow_name: &str) -> Result<String> {
    // Get the user's Documents directory
    let mut path = document_dir()
        .ok_or_else(|| IOError::new(ErrorKind::NotFound, "Documents directory not found"))?;
    
    // Append the required subdirectories and filename
    path.push("Anything");
    path.push("flows");
    path.push(flow_name); 
    path.push("flow.toml");

    // Check if the file exists
    if !path.exists() {
        return Err(IOError::new(ErrorKind::NotFound, format!("File not found at {:?}", path)));
    }

    // Read and return the file's contents
    fs::read_to_string(path)
}

//gets marked as done after it leaves here. Kinda a bad pattern i think
async fn execute_worker_task(app: &AppHandle, worker_type: &str, event_data: &HashMap<String, JsonValue>, event_id: &str) -> std::result::Result<(), String> {

    // Get values for eventProcessing Message
    // Use `.as_str().unwrap()` to extract the string value from the JsonValue
    let node_id = event_data.get("node_id").and_then(JsonValue::as_str).unwrap_or("");
    let flow_id = event_data.get("flow_id").and_then(JsonValue::as_str).unwrap_or("");
    let event_id = event_data.get("event_id").and_then(JsonValue::as_str).unwrap_or("");


    //write message 
    let message = format!("Executing Worker Task: {} for node_id: {} and flow_id: {} and event_id: {}", worker_type, node_id, flow_id, event_id);

    Event::EventProcessing { 
        message,
        event_id: event_id.to_string(),
        node_id: node_id.to_string(), 
        flow_id: flow_id.to_string(), 
         }.send(&app.get_window("main").unwrap()); 
   
    match worker_type {
        "start" => {
            // Do something for "start"
            // You can read the "flow_name" or other values from `event_data`
            if let Some(flow_name_value) = event_data.get("flow_name") {
                if let Some(flow_name_str) = flow_name_value.as_str() {
                    create_events_from_graph(app, flow_name_str).await;
                } else {
                    return Err("flow_name is not a string".to_string());
                }
            }
        },
        "rest" => {
            // Do something for "some_other_type"
            println!("Found a REST worker type");
            println!("{:?}", event_data); 
            // mark_as_done(app, event_id.to_string()).await;
            // println!("event_id: {} marked as COMPLETE", event_id);
        },
        // add other worker types here
        _ => {
            //FIXME: actually fail on unknown worker type
            println!("Worker type is not Start. Doing Work."); 
            //     //TODO: if node type is "START" create the other events
                // mark_as_done(app, event_id.to_string()).await;
                // println!("event_id: {} marked as COMPLETE", event_id);
            // return Err(format!("Unknown worker_type: {}", worker_type));
        }
    }

    Ok(())
}