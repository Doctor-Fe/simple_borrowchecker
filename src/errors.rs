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

#[derive(Debug)]
pub struct BracketError<'a> {
    bracket_type: &'a str
}

impl <'a>BracketError<'a> {
    pub fn new(bracket_type: &'a str) -> BracketError<'a> {
        BracketError { bracket_type }
    }
}

impl <'a>Display for BracketError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "There are no corresponding brackets to \"{}\"", self.bracket_type)
    }
}

impl <'a>Error for BracketError<'a> {}

#[derive(Debug)]
pub struct InvalidExpressionError;

impl Display for InvalidExpressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid expression detected.")
    }
}

impl Error for InvalidExpressionError {}

#[derive(Debug)]
pub struct NotImplementedError;

impl Display for NotImplementedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid expression detected.")
    }
}

impl Error for NotImplementedError {}