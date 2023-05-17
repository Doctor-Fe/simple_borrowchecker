use std::cmp::Ordering;
use std::collections::VecDeque;
use std::ops::{BitAnd, BitOr, BitXor, Shl, Shr};

use crate::parser::errors::{ParseError, ParseErrorType};

use super::VarType::{Integer, Uninitialized, Void};
use super::{ElementType, ExprParser, VarType};

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

    pub fn try_calculate_all(&mut self, mut data: (String, VecDeque<ElementType>)) -> Result<VarType, ParseError> {
        let f = if data.0.ends_with('=') {VecDeque::<ElementType>::pop_back} else {VecDeque::<ElementType>::pop_front};
        let mut num = f(&mut data.1).unwrap().to_vartype(&self)?;
        while let Some(d) = f(&mut data.1) {
            if (data.0 == "&&" && matches!(num, Integer(0))) || (data.0 == "||" && !matches!(num, Integer(0))) {
                break;
            }
            if data.0.ends_with('=') {
                num = self.calculate_binomial(&data.0, d, ElementType::Immediate(num))?;
            } else {
                num = self.calculate_binomial(&data.0, ElementType::Immediate(num), d)?;
            }
        }
        return Ok(num);
    }

    fn calculate_binomial(&mut self, op: &str, left: ElementType, right: ElementType) -> Result<VarType, ParseError> {
        match op {
            "+" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(p), Integer(q)) => Ok(Integer(p + q)),
                    (VarType::String(p), VarType::String(q)) => Ok(VarType::String(format!("{}{}", p, q))),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => return Err(ParseError::new(ParseErrorType::VoidOperation)),
                    _ => return Err(ParseError::new(ParseErrorType::InvalidExpression("Invalid operation.".to_string()))),
                }
            }),
            "-" => left.operation_number_failable(self, right, i32::checked_sub, &ParseErrorType::OperationUnderflow),
            "*" => left.operation_number_failable(self, right, i32::checked_mul, &ParseErrorType::OperationOverflow),
            "/" => left.operation_number_failable(self, right, i32::checked_div, &ParseErrorType::DivideByZero),
            "%" => left.operation_number_failable(self, right, i32::checked_rem, &ParseErrorType::DivideByZero),
            "|" => left.operation_number(self, right, i32::bitor),
            "&" => left.operation_number(self, right, i32::bitand),
            "^" => left.operation_number(self, right, i32::bitxor),
            ">>" => left.operation_number(self, right, i32::shr),
            "<<" => left.operation_number(self, right, i32::shl),
            "==" => left.operation(self, right, |a, b| Ok(Integer((a == b) as i32))),
            "!=" => left.operation(self, right, |a, b| Ok(Integer((a != b) as i32))),
            ">" => left.operation(self, right, |a, b| {
                match a.partial_cmp(&b) {
                    Some(a) => Ok(Integer(if a == Ordering::Greater {1} else {0})),
                    None => return Err(ParseError::new(ParseErrorType::InvalidExpression(format!("Cannot compare {} and {}.", a, b)))),
                }
            }),
            "<" => left.operation(self, right, |a, b| {
                match a.partial_cmp(&b) {
                    Some(a) => Ok(Integer(if a == Ordering::Less {1} else {0})),
                    None => return Err(ParseError::new(ParseErrorType::InvalidExpression(format!("Cannot compare {} and {}.", a, b)))),
                }
            }),
            "=>" => left.operation(self, right, |a, b| {
                match a.partial_cmp(&b) {
                    Some(a) => Ok(Integer(if a == Ordering::Less {0} else {1})),
                    None => return Err(ParseError::new(ParseErrorType::InvalidExpression(format!("Cannot compare {} and {}.", a, b)))),
                }
            }),
            "=<" => left.operation(self, right, |a, b| {
                match a.partial_cmp(&b) {
                    Some(a) => Ok(Integer(if a == Ordering::Greater {0} else {1})),
                    None => return Err(ParseError::new(ParseErrorType::InvalidExpression(format!("Cannot compare {} and {}.", a, b)))),
                }
            }),
            "&&" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => Ok(Integer(if a != 0 && b != 0 {1} else {0})),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => return Err(ParseError::new(ParseErrorType::VoidOperation)),
                    _ => return Err(ParseError::new(ParseErrorType::InvalidExpression("Invalid operation.".to_string()))),
                }
            }),
            "||" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => Ok(Integer(if a != 0 || b != 0 {1} else {0})),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => return Err(ParseError::new(ParseErrorType::VoidOperation)),
                    _ => return Err(ParseError::new(ParseErrorType::InvalidExpression("Invalid operation.".to_string()))),
                }
            }),
            "=" => left.operation_mut(self, right, |a, b| {Ok(*a = b)}),
            "+=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a = match a.checked_add(b) {
                        Some(a) => a,
                        None => return Err(ParseError::new(ParseErrorType::OperationOverflow))
                    },
                    (VarType::String(a), VarType::String(b)) => *a = format!("{}{}", a, b),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => return Err(ParseError::new(ParseErrorType::VoidOperation)),
                    _ => return Err(ParseError::new(ParseErrorType::InvalidExpression("Invalid operation.".to_string()))),
                }
                Ok(())
            }),
            "-=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a = match a.checked_sub(b) {
                        Some(a) => a,
                        None => return Err(ParseError::new(ParseErrorType::OperationUnderflow))
                    },
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => return Err(ParseError::new(ParseErrorType::VoidOperation)),
                    _ => return Err(ParseError::new(ParseErrorType::InvalidExpression("Invalid operation.".to_string()))),
                }
                Ok(())
            }),
            "*=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a = match a.checked_mul(b) {
                        Some(a) => a,
                        None => return Err(ParseError::new(ParseErrorType::OperationOverflow))
                    },
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => return Err(ParseError::new(ParseErrorType::VoidOperation)),
                    _ => return Err(ParseError::new(ParseErrorType::InvalidExpression("Invalid operation.".to_string()))),
                }
                Ok(())
            }),
            "/=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a = match a.checked_div(b) {
                        Some(a) => a,
                        None => return Err(ParseError::new(ParseErrorType::DivideByZero)),
                    },
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => return Err(ParseError::new(ParseErrorType::VoidOperation)),
                    _ => return Err(ParseError::new(ParseErrorType::InvalidExpression("Invalid operation.".to_string()))),
                }
                Ok(())
            }),
            "%=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a = match a.checked_rem(b) {
                        Some(a) => a,
                        None => return Err(ParseError::new(ParseErrorType::DivideByZero))
                    },
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => return Err(ParseError::new(ParseErrorType::VoidOperation)),
                    _ => return Err(ParseError::new(ParseErrorType::InvalidExpression("Invalid operation.".to_string()))),
                }
                Ok(())
            }),
            "|=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a |= b,
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => return Err(ParseError::new(ParseErrorType::VoidOperation)),
                    _ => return Err(ParseError::new(ParseErrorType::InvalidExpression("Invalid operation.".to_string()))),
                }
                Ok(())
            }),
            "&=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a &= b,
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => return Err(ParseError::new(ParseErrorType::VoidOperation)),
                    _ => return Err(ParseError::new(ParseErrorType::InvalidExpression("Invalid operation.".to_string()))),
                }
                Ok(())
            }),
            "^=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a ^= b,
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => return Err(ParseError::new(ParseErrorType::VoidOperation)),
                    _ => return Err(ParseError::new(ParseErrorType::InvalidExpression("Invalid operation.".to_string()))),
                }
                Ok(())
            }),
            ">>=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a = match a.checked_shr(b as u32) {
                        Some(a) => a,
                        None => return Err(ParseError::new(ParseErrorType::OperationUnderflow)),
                    },                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => return Err(ParseError::new(ParseErrorType::VoidOperation)),
                    _ => return Err(ParseError::new(ParseErrorType::InvalidExpression("Invalid operation.".to_string()))),
                }
                Ok(())
            }),
            "<<=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a = match a.checked_shl(b as u32) {
                        Some(a) => a,
                        None => return Err(ParseError::new(ParseErrorType::OperationOverflow)),
                    },
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => return Err(ParseError::new(ParseErrorType::VoidOperation)),
                    _ => return Err(ParseError::new(ParseErrorType::InvalidExpression("Invalid operation.".to_string()))),
                }
                Ok(())
            }),
            a => Err(ParseError::new(ParseErrorType::InvalidExpression(format!("Invalid operator \"{}\".", a)))),
        }
    }
}
