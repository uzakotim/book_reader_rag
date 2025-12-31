// pub fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
//     text
//         .chars()
//         .collect::<Vec<_>>()
//         .chunks(chunk_size)
//         .map(|c| c.iter().collect())
//         .collect()
// }
pub fn chunk_by_paragraph(text: &str, max_len: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for para in text.split("\n\n") {
        if current.len() + para.len() > max_len {
            if current.len() > 200 {
                chunks.push(current.clone());
            }
            current.clear();
        }

        current.push_str(para);
        current.push('\n');
    }

    if current.len() > 200 {
        chunks.push(current);
    }

    chunks
}