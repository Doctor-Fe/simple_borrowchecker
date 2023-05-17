use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct ParseError {
    error_type: ParseErrorType,
}

impl ParseError {
    pub fn new(data: ParseErrorType) -> Self {
        ParseError{ error_type: data }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_type)
    }
}

impl Error for ParseError {}

#[derive(Clone, Debug)]
pub enum ParseErrorType {
    Bracket(String),
    DivideByZero,
    InvalidDereference,
    InvalidExpression(String),
    InvalidInteger,
    MemoryLeak(usize, usize),
    OperationOverflow,
    OperationUnderflow,
    Unhandled,
    Uninitialized,
    VariableNotFound(String),
    VoidOperation,
}

impl Display for ParseErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseErrorType::Bracket(bracket_type) => write!(f, "There are no corresponding brackets to \"{}\"", bracket_type),
            ParseErrorType::DivideByZero => write!(f, "Divide by zero error"),
            ParseErrorType::InvalidDereference => write!(f, "Invalid dereference."),
            ParseErrorType::InvalidExpression(message) => write!(f, "Invalid expression: {}", message),
            ParseErrorType::InvalidInteger => write!(f, "Invalid integer."),
            ParseErrorType::OperationOverflow => write!(f, "Overflow occured"),
            ParseErrorType::OperationUnderflow => write!(f, "Underflow occured"),
            ParseErrorType::MemoryLeak(required, current) => write!(f, "{} bytes required but ensured memory was {} bytes.", required, current),
            ParseErrorType::Unhandled => write!(f, "Unhandled error."),
            ParseErrorType::Uninitialized => write!(f, "Variable was uninitialized"),
            ParseErrorType::VariableNotFound(variable) => write!(f, "Variable \"{}\" was not found.", variable),
            ParseErrorType::VoidOperation => write!(f, "Cannot operate with void."),
        }
    }
}

#[macro_export]
macro_rules! log_error {
    ($x: expr) => {
        log::error!("{}", $x)
    };
}

#[macro_export]
macro_rules! bracket_error {
    ($x: expr) => {
        {
            let e = ParseError::new(ParseErrorType::Bracket($x.to_string()));
            log::error!("{}", $x);
            return Err(e);
        }
    };
}
