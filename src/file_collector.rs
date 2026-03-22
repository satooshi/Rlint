use walkdir::WalkDir;

/// Compile exclude glob patterns once, warning on invalid entries.
pub fn compile_exclude_patterns(raw: &[String]) -> Vec<glob::Pattern> {
    raw.iter()
        .filter_map(|s| match glob::Pattern::new(s) {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("Warning: invalid exclude glob pattern '{}': {}", s, e);
                None
            }
        })
        .collect()
}

/// Strip the leading "./" prefix so that exclude glob patterns like
/// "vendor/**" match paths yielded as "./vendor/foo.rb".
pub fn normalize_path(raw: &str) -> &str {
    raw.strip_prefix("./").unwrap_or(raw)
}

pub fn collect_ruby_files(paths: &[String], exclude: &[glob::Pattern]) -> Vec<String> {
    let mut files = Vec::new();
    for path in paths {
        let meta = std::fs::metadata(path);
        if let Ok(m) = meta {
            if m.is_file() {
                let normalized = normalize_path(path);
                if !is_excluded(normalized, exclude) {
                    files.push(normalized.to_string());
                }
            } else {
                for entry in WalkDir::new(path)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let p = entry.path();
                    if p.is_file() {
                        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if ext == "rb"
                            || name == "Gemfile"
                            || name == "Rakefile"
                            || name.ends_with(".gemspec")
                            || name == "Guardfile"
                        {
                            let raw = p.to_string_lossy();
                            let path_str = normalize_path(&raw);
                            if !is_excluded(path_str, exclude) {
                                files.push(path_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    files.sort();
    files.dedup();
    files
}

/// Returns true if the path matches any pre-compiled exclude glob pattern.
pub fn is_excluded(path: &str, patterns: &[glob::Pattern]) -> bool {
    let path = std::path::Path::new(path);
    patterns.iter().any(|p| p.matches_path(path))
}
