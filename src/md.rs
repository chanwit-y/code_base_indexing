use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ExtractedFile {
    pub path: String,
    pub code: String,
}
/// Parses a markdown-formatted document and extracts file paths and their code blocks.
///
/// Expects the format:
/// ```
/// ### File N: path/to/file.ext
///
/// ```language
/// <code content>
/// ```
/// ```
pub fn extract_files(input: &str) -> Vec<ExtractedFile> {
    let mut results = Vec::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Match header like "### File 1: e2e/fixtures/mock-data.ts"
        if let Some(path) = parse_file_header(line) {
            // Scan forward for the next code fence opening
            i += 1;
            while i < lines.len() {
                let trimmed = lines[i].trim();
                if trimmed.starts_with("```") && trimmed.len() > 3 {
                    // Found opening fence (e.g. ```typescript), now collect until closing ```
                    i += 1;
                    let mut code_lines = Vec::new();
                    while i < lines.len() && lines[i].trim() != "```" {
                        code_lines.push(lines[i]);
                        i += 1;
                    }
                    results.push(ExtractedFile {
                        path: path.to_string(),
                        code: code_lines.join("\n"),
                    });
                    break;
                }
                i += 1;
            }
        }
        i += 1;
    }

    results
}

/// Extracts files and returns them as a HashMap keyed by file path.
pub fn extract_files_map(input: &str) -> HashMap<String, String> {
    extract_files(input)
        .into_iter()
        .map(|f| (f.path, f.code))
        .collect()
}

/// Parses a markdown header line to extract the file path.
/// e.g. "### File 1: e2e/fixtures/mock-data.ts" -> Some("e2e/fixtures/mock-data.ts")
fn parse_file_header(line: &str) -> Option<&str> {
    let line = line.strip_prefix("###")?;
    let line = line.trim();
    // Skip "File N:" prefix
    let (_, rest) = line.split_once(':')?;
    let path = rest.trim();
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}
