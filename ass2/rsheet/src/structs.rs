use std::collections::HashSet;

#[derive(Debug)]
pub enum Command {
    Get(String),
    Set(String, String),
    None,
}

#[derive(Debug)]
pub struct DependencyNode {
    pub formula: String,
    pub neighbors: HashSet<String>,
}

