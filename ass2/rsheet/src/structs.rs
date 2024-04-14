#[derive(Debug)]
pub enum Command {
    Get(String),
    Set(String, String),
    None,
}

pub struct DependencyNode {
    formula: String,
    neighbors: Vec<String>,
}

