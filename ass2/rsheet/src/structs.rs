
#[derive(Debug)]
pub enum Command {
    Get(String),
    Set(String, String),
    None,
}

