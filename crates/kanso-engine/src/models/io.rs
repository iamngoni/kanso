/// A file produced by Markdown export (relative path + content).
#[derive(Debug, Clone)]
pub struct ExportFile {
    pub path: String,
    pub content: String,
}

/// A file handed to Markdown import.
#[derive(Debug, Clone)]
pub struct ImportFile {
    pub filename: String,
    pub content: String,
}
