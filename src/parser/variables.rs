use log::trace;

use super::ExprParser;

impl ExprParser {
    /// 可変な状態で変数を取得します。
    /// - `name` - 変数名
    pub fn get_variable_mut(&mut self, name: &String) -> Option<&mut VarType> {
        self.variables.get_mut(name)
    }
    
    /// 変数を取得します。
    /// - `name` - 変数名
    pub fn get_variable(&self, name: &String) -> Option<&VarType> {
        self.variables.get(name)
    }

    /// 変数を作成します。
    /// - `name` - 新しく作成する変数名
    pub fn create_variable(&mut self, name: String){
        trace!("Variable \"{}\" was created.", name);
        if !self.variables.contains_key(&name) {
            self.variables.insert(name, VarType::Uninitialized);
        }
    }
}

/// 変数として格納可能な値を保持する構造体です。
#[derive(Debug, Clone)]
pub enum VarType {
    /// 未初期化の変数であることを表します。
    Uninitialized,
    /// 値がないことを表します。
    Void,
    /// 整数であることを表します。
    Integer(i32),
    /// 文字列であることを表します。
    String(String),
    /// ポインタであることを表します。
    Pointer(String),
}

impl VarType {
    /// 値が空であるかを取得する関数です。
    pub fn is_empty(&self) -> bool {
        match self {
            VarType::Uninitialized | VarType::Void => true,
            _ => false
        }
    }
}
