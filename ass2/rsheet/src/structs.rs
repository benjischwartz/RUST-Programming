use std::collections::HashSet;

#[derive(Debug)]
pub enum Command {
    Get(String),
    Set(String, String),
    None,
}

#[derive(Debug)]
pub struct DependencyNode {
    pub address: String,
    pub formula: String,
    pub neighbors: HashSet<String>,
}

