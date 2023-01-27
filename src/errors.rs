use std::{fmt::Display, error::Error};

/// 未定義の変数を参照しようとしたときのエラーです。
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

/// かっこの数が一致しないときのエラーです。
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

/// 無効な式が入力されたときのエラーです。
#[derive(Debug)]
pub struct InvalidExpressionError<'a> {
    message: &'a str
}

impl <'a>InvalidExpressionError<'a> {
    pub fn new(str: &'a str) -> InvalidExpressionError<'a> {
        InvalidExpressionError { message: str }
    }
}

impl <'a>Display for InvalidExpressionError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid expression detected.\n{}", self.message)
    }
}

impl <'a>Error for InvalidExpressionError<'a> {}

/// 入力がなかったときのエラーです。
#[derive(Debug)]
pub struct NoInputError;

impl Display for NoInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No inputs.")
    }
}

impl Error for NoInputError {}

/// voidと演算しようとしたときのエラーです。
#[derive(Debug)]
pub struct OperationError;

impl Display for OperationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cannot operate with void.")
    }
}

impl Error for OperationError {}

#[derive(Debug)]
pub struct ReferenceError {
    error_type: ReferenceErrorType
}

impl ReferenceError {
    pub fn invalid_dereference() -> Self {
        ReferenceError { error_type: ReferenceErrorType::InvalidDereference }
    }

    pub fn uninitialized() -> Self {
        ReferenceError { error_type: ReferenceErrorType::Uninitialized }
    }
}

impl Display for ReferenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_type)
    }
}

impl Error for ReferenceError {}

#[derive(Debug)]
pub enum ReferenceErrorType {
    InvalidDereference,
    Uninitialized,
}

impl Display for ReferenceErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ReferenceErrorType::InvalidDereference => "Invalid dereference.",
            ReferenceErrorType::Uninitialized => "Variable was uninitialized.",
        })
    }
}