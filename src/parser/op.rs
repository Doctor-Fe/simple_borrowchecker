use std::cmp::Ordering;
use std::ops::{BitOr, BitAnd, BitXor, Shl, Shr};
use std::{collections::VecDeque, error::Error};

use crate::{ret_err, parser::errors::{OperationError, OperationErrorType, InvalidExpressionError}};

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

    fn calculate_binomial(&mut self, op: &str, left: ElementType, right: ElementType) -> Result<VarType, Box<dyn Error>> {
        match op {
            "+" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(p), Integer(q)) => Ok(Integer(p + q)),
                    (VarType::String(p), VarType::String(q)) => Ok(VarType::String(format!("{}{}", p, q))),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError::new(OperationErrorType::WithVoid)),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "-" => left.operation_number_failable(self, right, i32::checked_sub),
            "*" => left.operation_number_failable(self, right, i32::checked_mul),
            "/" => left.operation_number_failable(self, right, i32::checked_div),
            "%" => left.operation_number_failable(self, right, i32::checked_rem),
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
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError::new(OperationErrorType::WithVoid)),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "||" => left.operation(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => Ok(Integer(if a != 0 || b != 0 {1} else {0})),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError::new(OperationErrorType::WithVoid)),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
            }),
            "=" => left.operation_mut(self, right, |a, b| {Ok(*a = b)}),
            "+=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a = match a.checked_add(b) {
                        Some(a) => a,
                        None => ret_err!(OperationError::new(OperationErrorType::Runtime)),
                    },
                    (VarType::String(a), VarType::String(b)) => *a = format!("{}{}", a, b),
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError::new(OperationErrorType::WithVoid)),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
                Ok(())
            }),
            "-=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a = match a.checked_sub(b) {
                        Some(a) => a,
                        None => ret_err!(OperationError::new(OperationErrorType::Runtime)),
                    },
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError::new(OperationErrorType::WithVoid)),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
                Ok(())
            }),
            "*=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a = match a.checked_mul(b) {
                        Some(a) => a,
                        None => ret_err!(OperationError::new(OperationErrorType::Runtime)),
                    },
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError::new(OperationErrorType::WithVoid)),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
                Ok(())
            }),
            "/=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a = match a.checked_div(b) {
                        Some(a) => a,
                        None => ret_err!(OperationError::new(OperationErrorType::Runtime)),
                    },
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError::new(OperationErrorType::WithVoid)),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
                Ok(())
            }),
            "%=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a = match a.checked_rem(b) {
                        Some(a) => a,
                        None => ret_err!(OperationError::new(OperationErrorType::Runtime)),
                    },
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError::new(OperationErrorType::WithVoid)),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
                Ok(())
            }),
            "|=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a |= b,
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError::new(OperationErrorType::WithVoid)),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
                Ok(())
            }),
            "&=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a &= b,
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError::new(OperationErrorType::WithVoid)),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
                Ok(())
            }),
            "^=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a ^= b,
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError::new(OperationErrorType::WithVoid)),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
                Ok(())
            }),
            ">>=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a = match a.checked_shr(b as u32) {
                        Some(a) => a,
                        None => ret_err!(OperationError::new(OperationErrorType::Runtime)),
                    },                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError::new(OperationErrorType::WithVoid)),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
                Ok(())
            }),
            "<<=" => left.operation_mut(self, right, |a, b| {
                match (a, b) {
                    (Integer(a), Integer(b)) => *a = match a.checked_shl(b as u32) {
                        Some(a) => a,
                        None => ret_err!(OperationError::new(OperationErrorType::Runtime)),
                    },
                    (Void | Uninitialized, _) | (_, Void | Uninitialized) => ret_err!(OperationError::new(OperationErrorType::WithVoid)),
                    _ => ret_err!(InvalidExpressionError::from("Invalid operation.")),
                }
                Ok(())
            }),
            a => ret_err!(InvalidExpressionError::new(format!("Invalid operator \"{}\".", a))),
        }
    }
}
