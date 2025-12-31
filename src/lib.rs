mod arxiv;
mod pdf;
mod chunk;
mod vector_store;
mod embedder;
mod llm;
mod retrieve;


use vector_store::VectorStore;
use vector_store::VectorEntry;
use embedder::embed;
use chunk::chunk_by_paragraph;




use pdf_extract::extract_text;

async fn pdf_book_to_text(path: &str) -> Result<String, String> {
    extract_text(path).map_err(|e| e.to_string())
}
fn clean_pdf_text(text: &str) -> String {
    text
        .lines()
        .filter(|l| l.len() > 30)      
        .collect::<Vec<_>>()
        .join("\n")
        .replace("-\n", "")
}
struct Chapter {
    title: String,
    content: String,
}
fn split_pdf_chapters(text: &str) -> Vec<Chapter> {
    let mut chapters = Vec::new();
    let mut current = String::new();
    let mut title = "Chapter".to_string();

    for line in text.lines() {
        if line.starts_with("CHAPTER ") || line.starts_with("Chapter ") {
            if !current.is_empty() {
                chapters.push(Chapter {
                    title: title.clone(),
                    content: current.clone(),
                });
                current.clear();
            }
            title = line.to_string();
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }

    if !current.is_empty() {
        chapters.push(Chapter { title, content: current });
    }

    chapters
}

use std::path::Path;

pub fn chunk_by_paragraph_with_overlap(
    text: &str,
    max_tokens: usize,
    overlap_tokens: usize,
) -> Vec<String> {
    let paragraphs: Vec<&str> = text
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();

    let mut chunks = Vec::new();
    let mut current = Vec::new();
    let mut current_tokens = 0;

    for para in paragraphs {
        let para_tokens = estimate_tokens(para);

        // If a single paragraph is too large, hard split it
        if para_tokens > max_tokens {
            let words: Vec<&str> = para.split_whitespace().collect();
            let mut start = 0;

            while start < words.len() {
                let end = (start + max_tokens).min(words.len());
                let slice = words[start..end].join(" ");
                chunks.push(slice);
                start = if end > overlap_tokens {
                    end - overlap_tokens
                } else {
                    end
                };
            }
            continue;
        }

        // If adding this paragraph exceeds limit, finalize current chunk
        if current_tokens + para_tokens > max_tokens {
            chunks.push(current.join("\n\n"));

            // Build overlap from end of previous chunk
            let overlap = build_overlap(&current, overlap_tokens);
            current = overlap;
            current_tokens = estimate_tokens(&current.join(" "));
        }

        current.push(para.to_string());
        current_tokens += para_tokens;
    }

    if !current.is_empty() {
        chunks.push(current.join("\n\n"));
    }

    chunks
}

fn estimate_tokens(text: &str) -> usize {
    // Rough heuristic: 1 token ≈ 0.75 words
    let words = text.split_whitespace().count();
    (words as f32 / 0.75).ceil() as usize
}

fn build_overlap(
    previous: &[String],
    overlap_tokens: usize,
) -> Vec<String> {
    let mut overlap = Vec::new();
    let mut tokens = 0;

    for para in previous.iter().rev() {
        let t = estimate_tokens(para);
        if tokens + t > overlap_tokens {
            break;
        }
        overlap.push(para.clone());
        tokens += t;
    }

    overlap.reverse();
    overlap
}

async fn ingest() -> Result<String, String> {
    let store = VectorStore::init_global();

    let book_path = Path::new("data/books/rag.pdf");

    // 1️⃣ Extract raw text from PDF
    let raw_text = extract_text(book_path)
        .map_err(|e| format!("PDF extract failed: {}", e))?;

    // 2️⃣ Clean PDF noise
    let cleaned_text = clean_pdf_text(&raw_text);

    // 3️⃣ Split into chapters (best-effort)
    let chapters = split_pdf_chapters(&cleaned_text);

    let mut total_chunks = 0;

    const MAX_CHARS: usize = 8000;
    const CHUNK_SIZE: usize = 700;
    const OVERLAP: usize = 100;

    for chapter in chapters {
        // Add chapter title as context
        let chapter_text = format!(
            "Chapter: {}\n\n{}",
            chapter.title,
            chapter.content
        );

        // Overlapping chunks
        let chunks = chunk_by_paragraph_with_overlap(
            &chapter_text,
            CHUNK_SIZE,
            OVERLAP,
        );

        for chunk in chunks {
            // HARD SAFETY CAP (critical)
            let safe_chunk = if chunk.len() > MAX_CHARS {
                &chunk[..MAX_CHARS]
            } else {
                &chunk
            };

            let embedding = match embed(safe_chunk).await {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("⚠️ Skipping chunk: {}", err);
                    continue;
                }
            };

            store.add(VectorEntry {
                embedding,
                text: safe_chunk.to_string(),
                section: chapter.title.clone(),
            });

            total_chunks += 1;
        }
    }


    Ok(format!(
        "📚 Book ingestion complete ({} chunks indexed)",
        total_chunks
    ))
}



async fn generate_idea() -> Result<String, String> {
    // 1. Build a retrieval-focused query
    let query = 
        "Unsolved problems, inefficiencies, and emerging needs";

    // 2. Embed & retrieve context
    let embedding = embedder::embed(&query)
        .await
        .map_err(|e| e.to_string())?;

    let context_chunks = retrieve::retrieve(embedding);

    // 3. Strong innovation-oriented prompt
    let prompt = format!(
    r#"
        You are a teaching expert.

        TASK:
        Generate a list of chapters in the book from the context

        RULES:
        - Be specific and practical
        - Follow the JSON OUTPUT FORMAT exactly
        - Do NOT add any explanations or extra text
        - Return ONLY valid JSON

        CONTEXT:
        {context}

        OUTPUT FORMAT (strict JSON):
        {{
        "chapters": [<string>, <string>, <string>,...]
        }}
        "#,
            context = context_chunks.join("\n---\n")
        );
    // 4. Run local LLM inference (blocking, but safe)
    let idea = llm::generate(&prompt).await.map_err(|e| e.to_string())?;

    Ok(idea)

}


pub async fn run() {
    let _ = fix_path_env::fix();
    match llm::start_ollama() {
        Ok(_) => (),
        Err(e) => panic!("Failed to start Ollama server: {}", e),   
    }
    match llm::load_model() {
        Ok(_) => (),
        Err(e) => panic!("Failed to load Ollama model: {}", e),   
    }
    println!("🚀 Starting rag app");
    println!("🌐 Ingesting data... {}", ingest().await.unwrap());
    match generate_idea().await {
        Ok(idea) => println!("{}", idea),
        Err(e) => panic!("Failed to generate idea: {}", e),
    }
}
