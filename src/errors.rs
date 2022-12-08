use std::{fmt::Display, error::Error};


#[derive(Debug)]
pub struct VariableNotFoundError {
    name: String,
}

impl VariableNotFoundError {
    pub fn new(name: String) -> VariableNotFoundError {
        VariableNotFoundError { name }
    }
}

impl Display for VariableNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Variable \"{}\" was not found.", self.name)
    }
}

impl Error for VariableNotFoundError {}
