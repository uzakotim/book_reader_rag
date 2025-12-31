pub async fn embed(text: &str) -> anyhow::Result<Vec<f32>> {
    let client = reqwest::Client::new();

    let res = client
        .post("http://localhost:11434/api/embeddings")
        .json(&serde_json::json!({
            "model": "nomic-embed-text",
            "prompt": text
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    // Ollama usually returns `embeddings: [[...]]`
    if let Some(arr) = res.get("embedding").and_then(|v| v.as_array()) {
        return Ok(arr.iter().map(|v| v.as_f64().unwrap() as f32).collect());
    }

    if let Some(arr) = res
        .get("embeddings")
        .and_then(|v| v.as_array())
        .and_then(|v| v.get(0))
        .and_then(|v| v.as_array())
    {
        return Ok(arr.iter().map(|v| v.as_f64().unwrap() as f32).collect());
    }

    Err(anyhow::anyhow!(
        "Invalid Ollama embedding response: {}",
        res
    ))
}
