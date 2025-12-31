pub async fn pdf_to_text(url: &str) -> anyhow::Result<String> {
    let bytes = reqwest::get(url).await?.bytes().await?;
    let text = pdf_extract::extract_text_from_mem(&bytes)?;
    Ok(text)
}
