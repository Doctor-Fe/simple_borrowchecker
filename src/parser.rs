use std::{collections::VecDeque, error::Error, fmt::Display};

use crate::pack;
use crate::ret_err;

use crate::evaluater::Evaluater;

pub struct ExprParser;

impl ExprParser {
    fn get_priority(op: &str) -> Option<i32> {
        if ["*", "/", "%"].contains(&op) {
            Some(2)
        } else if ["+", "-"].contains(&op) {
            Some(3)
        } else {
            None
        }
    }

    pub fn parse(eval: &mut Evaluater, data: &mut VecDeque<String>) -> Result<i32, Box<dyn Error>> {
        let mut list: Vec<(String, Vec<i32>)> = Vec::new();
        while !data.is_empty() {
            println!("{list:?}");
            let n = match data.pop_front() {
                Some(a) => match a.as_str() {
                    "(" => match Self::parse(eval, data) {
                        Ok(a) => a,
                        Err(a) => return Err(a),
                    },
                    ")" => {
                        break;
                    }
                    _ => match a.parse::<i32>() {
                        Ok(b) => b,
                        Err(_) => match eval.get_variable(&a.to_string()) {
                            Some(a) => *a,
                            None => match Self::get_priority(&a) {
                                Some(_) => ret_err!(VariableNotFoundError::new("Illegal operator.".to_string())),
                                None => ret_err!(VariableNotFoundError::new(a)),
                            },
                        },
                    },
                },
                None => unreachable!(),
            };
            if let Some(a) = list.pop() {
                match data.front() {
                    Some(b) => {
                        if Self::get_priority(b) > Self::get_priority(&a.0) {
                            let t = Self::calculate(&a.0, pack!([a.1, vec![n]].concat())).unwrap();
                            list.push((data.pop_front().unwrap(), vec![t]));
                        } else {
                            list.push(a);
                            list.push((data.pop_front().unwrap(), vec![n]));
                        }
                    }
                    None => {
                        list.push((a.0.clone(), [a.1.clone(), vec![n]].concat()));
                    }
                }
            } else {
                match data.pop_front() {
                    Some(a) => list.push((a, vec![n])),
                    None => return Ok(n),
                }
            }
        }
        let (mut a, mut b) = list.pop().unwrap();
        let mut num = Self::calculate(&a, pack!(b)).unwrap();
        while !list.is_empty() {
            let c = list.pop().unwrap();
            a = c.0;
            b = [c.1, vec![num]].concat();
            num = Self::calculate(&a, pack!(b)).unwrap();
        }
        return Ok(num);
    }

    fn calculate(op: &str, data: Box<dyn Iterator<Item = i32>>) -> Option<i32> {
        data.reduce(|a, b| match op {
            "+" => a + b,
            "-" => a - b,
            "*" => a * b,
            "/" => a / b,
            "%" => a % b,
            _ => unreachable!(),
        })
    }
}

#[macro_export]
macro_rules! ret_err {
    ($x: expr) => {
        return Err(Box::new($x))
    };
}

#[macro_export]
macro_rules! pack {
    ($x: expr) => {
        Box::new($x.into_iter())
    };
}

#[derive(Debug)]
struct VariableNotFoundError {
    name: String,
}

impl VariableNotFoundError {
    fn new(name: String) -> VariableNotFoundError {
        VariableNotFoundError { name }
    }
}

impl Display for VariableNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Variable \"{}\" was not found.", self.name)
    }
}

impl Error for VariableNotFoundError {}
