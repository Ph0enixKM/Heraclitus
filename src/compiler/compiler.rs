use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

use crate::rules::Rules;
use crate::compiler::lexer::Lexer;

#[derive(Clone, PartialEq)]
pub enum SeparatorMode {
    Manual,
    Automatic
}

#[derive(Clone, PartialEq)]
pub enum ScopingMode {
    Block,
    Indent
}

pub struct Compiler<AST> {
    pub name: String,
    pub rules: Rules,
    pub code: String,
    pub path: String,
    pub code_tree: HashMap<String, AST>,
    pub separator_mode: SeparatorMode,
    pub scoping_mode: ScopingMode
}

impl<AST> Compiler<AST> {
    pub fn new(name: &str, rules: Rules) -> Self {
        Compiler {
            name: String::from(name),
            rules,
            code: format!(""),
            path: format!("[code]"),
            code_tree: HashMap::new(),
            separator_mode: SeparatorMode::Automatic,
            scoping_mode: ScopingMode::Block
        }
    }

    pub fn load_file(mut self, file_path: String) -> std::io::Result<()> {
        let mut file = File::open(&file_path)?;
        file.read_to_string(&mut self.code)?;
        self.path = file_path;
        Ok(())
    }

    pub fn load<T: AsRef<str>>(&mut self, code: T) {
        self.code = String::from(code.as_ref());
    }

    pub fn set_path(&mut self, file_path: String) {
        self.path = file_path;
    }

    pub fn compile(&self) {
        let mut lexer = Lexer::new(&self);
        lexer.run();
    }
}
