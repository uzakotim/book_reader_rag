use quick_xml::Reader;
use quick_xml::events::Event;
pub struct Section {
    pub name: String,
    pub content: String,
}



/// Extracts PDF links from an arXiv Atom XML response
pub fn extract_pdf_links(xml: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut pdf_links = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"link" {
                    let mut href: Option<String> = None;
                    let mut link_type: Option<String> = None;

                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"href" => {
                                href = Some(
                                    String::from_utf8_lossy(&attr.value).to_string()
                                );
                            }
                            b"type" => {
                                link_type = Some(
                                    String::from_utf8_lossy(&attr.value).to_string()
                                );
                            }
                            _ => {}
                        }
                    }

                    if let (Some(href), Some(link_type)) = (href, link_type) {
                        if link_type == "application/pdf" {
                            pdf_links.push(href);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("XML parse error: {:?}", e);
                break;
            }
            _ => {}
        }

        buf.clear();
    }

    pdf_links
}

pub fn extract_sections(text: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current = Section {
        name: "unknown".into(),
        content: String::new(),
    };

    for line in text.lines() {
        let lower = line.to_lowercase();

        if lower.contains("abstract") {
            push_section(&mut sections, &mut current);
            current.name = "abstract".into();
        } else if lower.contains("introduction") {
            push_section(&mut sections, &mut current);
            current.name = "introduction".into();
        } else if lower.contains("limitations") {
            push_section(&mut sections, &mut current);
            current.name = "limitations".into();
        } else if lower.contains("future work") {
            push_section(&mut sections, &mut current);
            current.name = "future_work".into();
        } else {
            current.content.push_str(line);
            current.content.push('\n');
        }
    }

    push_section(&mut sections, &mut current);
    sections
}

pub fn push_section(sections: &mut Vec<Section>, current: &mut Section) {
    if current.content.len() > 200 {
        sections.push(Section {
            name: current.name.clone(),
            content: current.content.clone(),
        });
    }
    current.content.clear();
}

pub fn is_innovation_section(name: &str) -> bool {
    matches!(
        name,
        "abstract" | "introduction" | "limitations" | "future_work"
    )
}