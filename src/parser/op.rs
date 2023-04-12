use std::cmp::Ordering;
use std::{collections::VecDeque, error::Error};

use crate::parser::errors::OperationError;
use crate::{ret_err, parser::errors::InvalidExpressionError};

use super::{ExprParser, ElementType, VarType};
use super::VarType::{Integer, Void, Uninitialized};

impl ExprParser {
    /// 二項演算子の優先順位を返します。
    /// * `op` - 優先順位を取得する演算子
    pub fn get_priority(op: &str) -> Option<usize> {
        match op {
            "*" | "/" | "%" => Some(0),
            "+" | "-" => Some(1),
            ">>" | "<<" => Some(2),
            "&" | "|" | "^" => Some(3),
            "==" | "!=" | ">" | "<" | ">=" | "<=" => Some(4),
            "&&" | "||" => Some(5),
            "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "|=" | "&=" | "^=" | ">>=" | "<<=" => Some(6),
            _ => None,
        }
    }

    /// 単項演算子か判定する関数です。
    /// - `op` - 判定する演算子
    pub fn is_monomial(op: &str) -> bool {
        const OPERATORS: [&str; 7] = ["+", "-", "&", "&&", "*", "!", "~"];
        return OPERATORS.contains(&op);
    }

    pub fn try_calculate_all(&mut self, mut data: (String, VecDeque<ElementType>)) -> Result<VarType, Box<dyn Error>> {
        if data.0 != "=" {
            let mut num = data.1.pop_front().unwrap();
            while !data.1.is_empty() {
                if data.0 == "&&" {
                    if matches!(num,  ElementType::Immediate(Integer(0))) {
                        break;
                    }
                } else if data.0 == "||" {
                    if !matches!(num,  ElementType::Immediate(Integer(0))) {
                        break;
                    }
                }
                let d = data.1.pop_front().unwrap();
                num = ElementType::Immediate(self.calculate_binomial(&data.0, num, d)?);
            }
            return num.to_vartype(self);
        } else {
            let mut num = data.1.pop_back().unwrap();
            while !data.1.is_empty() {
                let d = data.1.pop_back().unwrap();
                num = ElementType::Immediate(self.calculate_binomial(&data.0, d, num)?);
            }
            return num.to_vartype(self);
        }
    }

    fn calculate_binomial(&mut self, op: &str, left: ElementType, right: ElementType) -> Result<VarType, Box<dyn Error>> {
        match op {
            "+" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(p), Integer(q)) => Ok(Integer(p + q)),
                    (VarType::String(p), VarType::String(q)) => Ok(VarType::String(format!("{}{}", p, q))),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "-" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(p), Integer(q)) => Ok(Integer(p - q)),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "*" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(p), Integer(q)) => Ok(Integer(p * q)),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "/" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => Ok(Integer(a / b)),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "%" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => Ok(Integer(a % b)),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "|" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => Ok(Integer(a | b)),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "&" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => Ok(Integer(a & b)),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "^" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => Ok(Integer(a ^ b)),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            ">>" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => Ok(Integer(a >> b)),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "<<" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => Ok(Integer(a << b)),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "==" => left.operation(self, right, |a, b| Ok(Integer(if a == b {1} else {0}))),
            "!=" => left.operation(self, right, |a, b| Ok(Integer(if a != b {1} else {0}))),
            ">" => left.operation(self, right, |a, b| {
                match a.partial_cmp(&b) {
                    Some(a) => Ok(Integer(if a == Ordering::Greater {1} else {0})),
                    None => ret_err!(InvalidExpressionError::new(format!("Cannot compare {} and {}.", a, b))),
                }
            }),
            "<" => left.operation(self, right, |a, b| {
                match a.partial_cmp(&b) {
                    Some(a) => Ok(Integer(if a == Ordering::Less {1} else {0})),
                    None => ret_err!(InvalidExpressionError::new(format!("Cannot compare {} and {}.", a, b))),
                }
            }),
            "=>" => left.operation(self, right, |a, b| {
                match a.partial_cmp(&b) {
                    Some(a) => Ok(Integer(if a == Ordering::Less {0} else {1})),
                    None => ret_err!(InvalidExpressionError::new(format!("Cannot compare {} and {}.", a, b))),
                }
            }),
            "=<" => left.operation(self, right, |a, b| {
                match a.partial_cmp(&b) {
                    Some(a) => Ok(Integer(if a == Ordering::Greater {0} else {1})),
                    None => ret_err!(InvalidExpressionError::new(format!("Cannot compare {} and {}.", a, b))),
                }
            }),
            "&&" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => Ok(Integer(if a != 0 && b != 0 {1} else {0})),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "||" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => Ok(Integer(if a != 0 || b != 0 {1} else {0})),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "=" => left.operation_mut(self, right, |a, b| {*a = b; Ok(Void)}),
            "+=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a += b,
                    (VarType::String(a), VarType::String(b)) => *a = format!("{}{}", a, b),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
                return Ok(Void);
            }),
            "-=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a -= b,
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
                return Ok(Void);
            }),
            "*=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => {*a *= b; Ok(Void)},
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "/=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => {*a /= b; Ok(Void)},
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "%=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => {*a %= b; Ok(Void)},
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "|=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => {*a |= b; Ok(Void)},
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "&=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => {*a &= b; Ok(Void)},
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "^=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => {*a ^= b; Ok(Void)},
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            ">>=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => {*a >>= b; Ok(Void)},
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "<<=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => {*a <<= b; Ok(Void)},
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            a => ret_err!(InvalidExpressionError::new(format!("Invalid operator \"{}\".", a))),
        }
    }
}
