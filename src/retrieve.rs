use crate::vector_store::{VectorEntry, VectorStore};

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let mag_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (mag_a * mag_b + 1e-6)
}

/// Retrieve semantically relevant book knowledge
pub fn retrieve(query_embedding: Vec<f32>) -> Vec<String> {
    let store = VectorStore::global();
    let entries = store.all();

    // 1️⃣ Score purely by cosine similarity
    let mut scored: Vec<(f32, &VectorEntry)> = entries
        .iter()
        .map(|e| {
            let sim = cosine_similarity(&query_embedding, &e.embedding);
            (sim, e)
        })
        .collect();

    // 2️⃣ Sort by relevance
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    // 3️⃣ Diversity filter (avoid near-duplicates)
    let mut results = Vec::new();
    let mut selected: Vec<&VectorEntry> = Vec::new();

    for (_, entry) in scored {
        if results.len() >= 6 {
            break;
        }

        // Skip near-duplicate chunks
        if selected.iter().any(|e| {
            cosine_similarity(&e.embedding, &entry.embedding) > 0.85
        }) {
            continue;
        }

        selected.push(entry);
        results.push(entry.text.clone());
    }

    results
}
