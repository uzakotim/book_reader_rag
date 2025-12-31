use anyhow::Result;
use once_cell::sync::OnceCell;
use std::process::{Command};

use serde::{Deserialize, Serialize};

static SYSTEM_PROMPT: OnceCell<String> = OnceCell::new();

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    system: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}


pub fn start_ollama() -> anyhow::Result<()> {

    // let ollama_bin = app
    //     .path()
    //     .resource_dir()
    //     .unwrap()
    //     .join("ollama");

    // 1️⃣ Start server FIRST
    Command::new("ollama")
        .arg("serve")
        .spawn()?;

    // 2️⃣ Wait briefly for server to be ready
    std::thread::sleep(std::time::Duration::from_secs(1));

    Ok(())
}

pub fn load_model() -> Result<()> {
    SYSTEM_PROMPT.set(
        "The assistant will act like a helpful research assistant.".to_string()
    ).ok();
    Ok(())
}

pub async fn generate(prompt: &str) -> Result<String> {
    let system = SYSTEM_PROMPT.get().expect("Model not loaded");

    let req = OllamaRequest {
        model: "gemma2:2b",
        prompt,
        system,
        stream: false,
    };

    let client = reqwest::Client::new();
    let res = client
        .post("http://127.0.0.1:11434/api/generate")
        .json(&req)
        .send()
        .await?
        .json::<OllamaResponse>()
        .await?;

    Ok(res.response)
}