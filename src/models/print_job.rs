use serde::{Serialize, Deserialize};

#[allow(dead_code)] // Add this line to silence the warning
#[derive(Debug, Serialize, Deserialize)]
pub struct PrintJob {
    pub access_code: String,
    pub file_paths: Vec<String>,
    pub copies: i32,
    pub color: bool,
    pub paper_size: String,
    pub orientation: String,
    pub print_sides: String,
    pub page_selection: String,
    pub custom_page_range: String,
    pub print_quality: String,
    pub scaling: String,
    pub expiry_minutes: i32,
    pub master_settings: Option<String>,
}