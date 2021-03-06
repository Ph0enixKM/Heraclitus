use crate::compiler::logger::ErrorDetails;

/// Result that should be returned in the parsing phase
pub type SyntaxResult = Result<(), ErrorDetails>;

/// Trait for parsing
/// 
/// Trait that should be implemented in order to parse tokens with heraklit
pub trait SyntaxModule<M> {
    /// Create a new default implementation of syntax module
    fn new() -> Self;
    /// Parse and create AST
    /// 
    /// This method is fundamental in creating a functional AST node that can determine 
    /// if tokens provided by metadata can be consumed to create this particular AST node.
    fn parse(&mut self, meta: &mut M) -> SyntaxResult;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compiler::parser::pattern::*;
    use crate::compiler::parser::preset::*;
    use crate::compiler::{ Token, DefaultMetadata, Metadata };

    struct Expression {}
    impl SyntaxModule<DefaultMetadata> for Expression {
        fn new() -> Self {
            Expression { }
        }
        fn parse(&mut self, meta: &mut DefaultMetadata) -> SyntaxResult {
            token(meta, "let")?;
            Ok(())
        }
    }

    #[test]
    fn test_token_match() {
        let mut exp = Expression {};
        let dataset1 = vec![
            Token {
                word: format!("let"),
                pos: (0, 0)
            }
        ];
        let dataset2 = vec![
            Token {
                word: format!("tell"),
                pos: (0, 0)
            }
        ];
        let path = Some(format!("path/to/file"));
        let result1 = exp.parse(&mut DefaultMetadata::new(dataset1, path.clone()));
        let result2 = exp.parse(&mut DefaultMetadata::new(dataset2, path.clone()));
        assert!(result1.is_ok());
        assert!(result2.is_err());
    }

    struct Preset {}
    impl SyntaxModule<DefaultMetadata> for Preset {
        fn new() -> Self {
            Preset {  }
        }
        fn parse(&mut self, meta: &mut DefaultMetadata) -> SyntaxResult {
            variable(meta, vec!['_'])?;
            numeric(meta, vec![])?;
            number(meta, vec![])?;
            integer(meta, vec![])?;
            float(meta, vec![])?;
            Ok(())
        }
    }

    #[test]
    fn test_preset_match() {
        let mut exp = Preset {};
        let dataset = vec![
            // Variable
            Token { word: format!("_text"), pos: (0, 0) },
            // Numeric
            Token { word: format!("12321"), pos: (0, 0) },
            // Number
            Token { word: format!("-123.12"), pos: (0, 0) },
            // Integer
            Token { word: format!("-12"), pos: (0, 0) },
            // Float
            Token { word: format!("-.681"), pos: (0, 0)}
        ];
        let path = Some(format!("path/to/file"));
        let result = exp.parse(&mut DefaultMetadata::new(dataset, path));
        assert!(result.is_ok());
    }

    struct PatternModule {}
    impl SyntaxModule<DefaultMetadata> for PatternModule {
        fn new() -> Self {
            PatternModule {  }
        }
        #[allow(unused_must_use)]
        fn parse(&mut self, meta: &mut DefaultMetadata) -> SyntaxResult {
            // Any
            if let Ok(_) = token(meta, "apple") {}
            else if let Ok(_) = token(meta, "orange") {}
            else if let Ok(_) = token(meta, "banana") {}
            else { 
                if let Err(details) = token(meta, "banana") {
                    return Err(details);
                }
            }
            // Optional
            token(meta, "optional");
            // Syntax
            syntax(meta, &mut Expression::new())?;
            // Repeat
            loop {
                if let Err(_) = token(meta, "test") {
                    break;
                }
                if let Err(_) = token(meta, ",") {
                    break;
                }
            }
            // End token
            token(meta, "end");
            Ok(())
        }
    }

    #[test]
    fn rest_match() {
        let mut exp = PatternModule {};
        // Everything should pass
        let dataset1 = vec![
            Token { word: format!("orange"), pos: (0, 0) },
            Token { word: format!("optional"), pos: (0, 0) },
            Token { word: format!("let"), pos: (0, 0) },
            Token { word: format!("this"), pos: (0, 0) },
            Token { word: format!(","), pos: (0, 0) },
            Token { word: format!("this"), pos: (0, 0) },
            Token { word: format!("end"), pos: (0, 0) }
        ];
        // Token should fail
        let dataset2 = vec![
            Token { word: format!("kiwi"), pos: (0, 0) },
            Token { word: format!("optional"), pos: (0, 0) },
            Token { word: format!("let"), pos: (0, 0) },
            Token { word: format!("this"), pos: (0, 0) },
            Token { word: format!(","), pos: (0, 0) },
            Token { word: format!("this"), pos: (0, 0) },
            Token { word: format!("end"), pos: (0, 0) }
        ];
        // Syntax should fail
        let dataset3 = vec![
            Token { word: format!("orange"), pos: (0, 0) },
            Token { word: format!("tell"), pos: (0, 0) },
            Token { word: format!("this"), pos: (0, 0) },
            Token { word: format!(","), pos: (0, 0) },
            Token { word: format!("this"), pos: (0, 0) },
            Token { word: format!("end"), pos: (0, 0) }
        ];
        // Token should fail because of repeat matching (this , this) ,
        let dataset4 = vec![
            Token { word: format!("orange"), pos: (0, 0) },
            Token { word: format!("tell"), pos: (0, 0) },
            Token { word: format!("this"), pos: (0, 0) },
            Token { word: format!(","), pos: (0, 0) },
            Token { word: format!("this"), pos: (0, 0) },
            Token { word: format!("this"), pos: (0, 0) },
            Token { word: format!("end"), pos: (0, 0) }
        ];
        let path = Some(format!("path/to/file"));
        let result1 = exp.parse(&mut DefaultMetadata::new(dataset1, path.clone()));
        let result2 = exp.parse(&mut DefaultMetadata::new(dataset2, path.clone()));
        let result3 = exp.parse(&mut DefaultMetadata::new(dataset3, path.clone()));
        let result4 = exp.parse(&mut DefaultMetadata::new(dataset4, path.clone()));
        assert!(result1.is_ok());
        assert!(result2.is_err());
        assert!(result3.is_err());
        assert!(result4.is_err());
    }

}
