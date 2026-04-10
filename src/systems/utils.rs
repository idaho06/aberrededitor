/// Convert an absolute path to a path relative to the current working directory.
/// Works across directory boundaries (produces `../` traversals when needed).
/// Falls back to the original path if canonicalization fails.
pub fn to_relative(path: &str) -> String {
    let make_relative = || -> Option<String> {
        let canon_path = std::path::Path::new(path).canonicalize().ok()?;
        let canon_cwd = std::env::current_dir().ok()?.canonicalize().ok()?;

        let path_parts: Vec<_> = canon_path.components().collect();
        let cwd_parts: Vec<_> = canon_cwd.components().collect();

        let common = path_parts
            .iter()
            .zip(cwd_parts.iter())
            .take_while(|(a, b)| a == b)
            .count();

        let mut result = std::path::PathBuf::new();
        for _ in 0..(cwd_parts.len() - common) {
            result.push("..");
        }
        for part in &path_parts[common..] {
            result.push(part);
        }
        Some(result.to_string_lossy().into_owned())
    };
    make_relative().unwrap_or_else(|| path.to_owned())
}
