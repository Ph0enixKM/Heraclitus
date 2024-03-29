use heraclitus_compiler::prelude::*;
use super::text::*;

#[derive(Debug)]
pub enum ExprType {
    Text(Text)
}

#[derive(Debug)]
pub struct Expr {
    expr: Option<Box<ExprType>>
}
impl Expr {
    fn get<M,S>(&mut self, meta: &mut M, mut module: S, cb: impl Fn(S) -> ExprType) -> SyntaxResult
    where
        M: Metadata,
        S: SyntaxModule<M>
    {
        match syntax(meta, &mut module) {
            Ok(()) => {
                self.expr = Some(Box::new(cb(module)));
                Ok(())    
            }
            Err(details) => Err(details)
        }
    }
    fn parse_module(&mut self, meta: &mut DefaultMetadata, module: ExprType) -> SyntaxResult {
        match module {
            ExprType::Text(md) => self.get(meta, md, ExprType::Text)
        }
    }
}
impl SyntaxModule<DefaultMetadata> for Expr {
    syntax_name!("Expr");
    fn new() -> Self {
        Expr { expr: None }
    }
    fn parse(&mut self, meta: &mut DefaultMetadata) -> SyntaxResult {
        let modules: Vec<ExprType> = vec![
            ExprType::Text(Text::new())
        ];
        let mut err = None;
        for module in modules {
            match self.parse_module(meta, module) {
                Ok(()) => return Ok(()),
                Err(details) => {
                    err = Some(details);
                }
            }
        }
        Err(err.unwrap())
    }
}