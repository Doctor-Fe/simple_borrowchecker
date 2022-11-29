use std::collections::{HashMap, VecDeque};

pub struct Evaluater {
    variables: HashMap<String, i32>,
    cmds: Vec<Vec<String>>,
}

impl Evaluater {
    pub fn new() -> Evaluater {
        Evaluater {
            variables: HashMap::new(),
            cmds: Vec::new(),
        }
    }

    pub fn split_elements(&mut self, data: String) {
        let mut word: String = String::new();
        let mut cmd: Vec<String> = Vec::new();
        for a in data.chars() {
            if ['\r', '\n', '\t', ' '].contains(&a) {
                if !word.is_empty() {
                    cmd.push(word.clone());
                    word.clear();
                }
            } else if ['+', '-', '/', '*', '%', '(', ')', '='].contains(&a) {
                if !word.is_empty() {
                    cmd.push(word.clone());
                    word.clear();
                }
                cmd.push(a.to_string());
            } else if a == ';' {
                if !word.is_empty() {
                    cmd.push(word.clone());
                    word.clear();
                }
                if !cmd.is_empty() {
                    self.cmds.push(cmd.clone());
                }
            } else {
                word.push(a);
            }
        }
    }

    pub fn pop_command(&mut self) -> Option<VecDeque<String>> {
        match self.cmds.pop() {
            Some(a) => Some(a.into_iter().collect::<VecDeque<String>>()),
            None => None,
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<&i32> {
        return self.variables.get(name);
    }

    pub fn get_variable_mut(&mut self, name: &str) -> Option<&mut i32> {
        return self.variables.get_mut(name);
    }

    pub fn create_variable(&mut self, name: &str) -> bool {
        if self.variables.contains_key(name) {
            return false;
        } else {
            self.variables.insert(name.to_string(), 0);
            return true;
        }
    }
}
