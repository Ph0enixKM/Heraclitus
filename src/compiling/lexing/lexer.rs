use crate::compiling::{ Compiler, Token, SeparatorMode, ScopingMode };
use super::compound_handler::{CompoundHandler, CompoundReaction};
use super::region_handler::{ RegionHandler, RegionReaction };
use super::reader::Reader;
use crate::compiling::failing::position_info::PositionInfo;

// This is just an estimation of token amount
// inside of a typical 200-lined file.
const AVG_TOKEN_AMOUNT: usize = 1024;

/// Lexer's error type
#[derive(Debug)]
pub enum LexerErrorType {
    /// Unspillable region has been spilled
    Singleline,
    /// Given region left unclosed
    Unclosed
}

/// Type containing full error of lexer
pub type LexerError = (LexerErrorType, PositionInfo);

/// The Lexer
///
/// Lexer takes source code in a form of a string and translates it to a list of tokens.
/// This particular implementation requires additional metadata such as like regions or symbols.
/// These can be supplied by the `Compiler` in a one cohesive package. Hence the API requires to
/// pass a reference to the `Compiler`.
pub struct Lexer<'a> {
    symbols: Vec<char>,
    escape_symbol: char,
    compound: CompoundHandler,
    region: RegionHandler,
    reader: Reader<'a>,
    path: Option<String>,
    /// This attribute stores parsed tokens by the lexer
    pub lexem: Vec<Token>,
    separator_mode: SeparatorMode,
    scoping_mode: ScopingMode,
    is_escaped: bool,
    position: (usize, usize),
    index: usize,
    token_start_index: usize
}

impl<'a> Lexer<'a> {
    /// Create a new Lexer based on the compiler metadata
    pub fn new(cc: &'a Compiler) -> Self {
        let code: &'a String = cc.code.as_ref().unwrap();
        Lexer {
            symbols: cc.rules.symbols.clone(),
            escape_symbol: cc.rules.escape_symbol,
            compound: CompoundHandler::new(&cc.rules),
            region: RegionHandler::new(&cc.rules),
            reader: Reader::new(code),
            path: cc.path.clone(),
            lexem: Vec::with_capacity(AVG_TOKEN_AMOUNT),
            separator_mode: cc.separator_mode.clone(),
            scoping_mode: cc.scoping_mode.clone(),
            is_escaped: false,
            position: (0, 0),
            index: 0,
            token_start_index: 0
        }
    }

    /// Add indentation to the lexem
    #[inline]
    fn add_indent(&mut self, word: String) -> String {
        if !word.is_empty() {
            // Getting position by word here would attempt to
            // substract with overflow since the new line character
            // technically belongs to the previous line
            let (row, _col) = self.reader.get_position();
            self.lexem.push(Token {
                word,
                pos: (row, 1),
                start: self.token_start_index,
            });
            self.position = (0, 0);
            String::new()
        } else { word }
    }

    /// Add word that has been completed in previous iteration to the lexem
    #[inline]
    fn add_word(&mut self, word: String) -> String {
        if !word.is_empty() {
            self.lexem.push(Token {
                word,
                pos: self.position,
                start: self.token_start_index
            });
            self.position = (0, 0);
            String::new()
        }
        else { word }
    }

    /// Add word that has been completed in current iteration to the lexem
    #[inline]
    fn add_word_inclusively(&mut self, word: String) -> String {
        if !word.is_empty() {
            self.lexem.push(Token {
                word,
                pos: self.position,
                start: self.token_start_index
            });
            self.position = (0, 0);
            String::new()
        }
        else { word }
    }

    /// Checks whether this is a nontokenizable region
    #[inline]
    pub fn is_tokenized_region(&self, reaction: &RegionReaction) -> bool {
        if let Some(region) = self.region.get_region().as_ref() {
            region.tokenize && *reaction == RegionReaction::Pass
        }
        else { false }
    }

    /// Pattern code for adding a symbol
    /// **[*]**
    #[inline]
    fn pattern_add_symbol(&mut self, mut word: String, letter: char) -> String {
        word = self.add_word(word);
        if word.is_empty() {
            self.token_start_index = self.index;
        }
        self.word_push(&mut word, letter);
        self.position = self.reader.get_position();
        self.add_word_inclusively(word)
    }

    /// Pattern code for beginning a new region
    /// **[**
    #[inline]
    fn pattern_begin(&mut self, mut word: String, letter: char) -> String {
        word = self.add_word(word);
        self.word_push(&mut word, letter);
        word
    }

    /// Pattern code for ending current region
    /// **]**
    #[inline]
    fn pattern_end(&mut self, mut word: String, letter: char) -> String {
        self.word_push(&mut word, letter);
        self.add_word_inclusively(word)
    }

    /// Push letter to the word and set token start index
    fn word_push(&mut self, word: &mut String, letter: char) {
        if word.is_empty() {
            self.token_start_index = self.index;
        }
        word.push(letter);
    }

    /// Tokenize source code
    ///
    /// Run lexer and tokenize code. The result is stored in the lexem attribute
    pub fn run(&mut self) -> Result<(), LexerError> {
        let mut word = String::new();
        let mut is_indenting = false;
        while let Some(letter) = self.reader.next() {
            self.index = self.reader.get_index();

            /****************/
            /* Set Position */
            /****************/

            // If the new position hasn't been set yet, set it
            if self.position == (0, 0) {
                // If separator mode is set to Manual and the letter is a separator,
                // then skip finding a new position
                if SeparatorMode::Manual != self.separator_mode || letter != '\n' {
                    let region = self.region.get_region().unwrap();
                    // If the region is tokenized, then check if the letter is a separator
                    if !region.tokenize || !vec![' ', '\t'].contains(&letter) {
                        self.position = self.reader.get_position();
                    }
                }
            }

            // Reaction stores the reaction of the region handler
            // Have we just opened or closed some region?
            let reaction = self.region.handle_region(&self.reader, self.is_escaped);
            match reaction {
                // If the region has been opened
                // Finish the part that we have been parsing
                RegionReaction::Begin(tokenize) => {
                    // Also if the new region is an interpolation that tokenizes
                    // the inner content - separate the region from the content
                    if tokenize {
                        word = self.pattern_add_symbol(word, letter);
                    }
                    // Regular region case
                    else {
                        // This is supposed to prevent overshadowing new line
                        // character if region rule opens with newline
                        if letter == '\n' {
                            // This additionally creates a new token
                            word = self.pattern_add_symbol(word, letter);
                        }
                        // Normally start a new region
                        word = self.pattern_begin(word, letter);
                    }
                },
                // If the region has been closed
                // Add the closing region and finish the word
                RegionReaction::End(tokenize) => {
                    // Also if the new region is an interpolation that tokenizes
                    // the inner content - separate the region from the content
                    if tokenize {
                        word = self.pattern_add_symbol(word, letter);
                    }
                    // Regular region case
                    else {
                        // Normally close the region
                        word = self.pattern_end(word, letter);
                        // This is supposed to prevent overshadowing new line
                        // character if region rule closes with newline
                        if letter == '\n' {
                            // This additionally creates a new token
                            word = self.pattern_add_symbol(word, letter);
                        }
                    }
                }
                RegionReaction::Pass => {
                    match self.compound.handle_compound(letter, &self.reader, self.is_tokenized_region(&reaction)) {
                        CompoundReaction::Begin => word = self.pattern_begin(word, letter),
                        CompoundReaction::Keep => self.word_push(&mut word, letter),
                        CompoundReaction::End => word = self.pattern_end(word, letter),
                        CompoundReaction::Pass => {
                            // Handle region scope
                            if !self.is_tokenized_region(&reaction) {
                                let region = self.region.get_region().unwrap();
                                // Flip escaped key
                                self.is_escaped = (!self.is_escaped && letter == self.escape_symbol)
                                    .then(|| !self.is_escaped)
                                    .unwrap_or(false);
                                // Handle singleline attribute
                                if letter == '\n' && region.singleline {
                                    let pos = self.reader.get_position();
                                    return Err((
                                        LexerErrorType::Singleline,
                                        PositionInfo::at_pos(self.path.clone(), pos, 0).data(region.name.clone())
                                    ))
                                }
                                self.word_push(&mut word, letter);
                            }
                            else {

                                /******************/
                                /* Mode modifiers */
                                /******************/

                                // Create indent regions: '\n   '
                                if let ScopingMode::Indent = self.scoping_mode {
                                    // If we are still in the indent region - proceed
                                    if is_indenting && vec![' ', '\t'].contains(&letter) {
                                        self.word_push(&mut word, letter);
                                    }
                                    // If it's the new line - start indent region
                                    if letter == '\n' {
                                        is_indenting = true;
                                        word = self.pattern_begin(word, letter);
                                    }
                                    // Check if the current letter
                                    // concludes current indent region
                                    if is_indenting {
                                        if let Some(next_char) = self.reader.peek() {
                                            if !vec![' ', '\t'].contains(&next_char) {
                                                word = self.add_indent(word);
                                                is_indenting = false;
                                            }
                                        }
                                        continue
                                    }
                                }
                                // Skip newline character if we want to manually insert semicolons
                                if let SeparatorMode::Manual = self.separator_mode {
                                    if letter == '\n' {
                                        word = self.add_word(word);
                                        continue
                                    }
                                }

                                /*****************/
                                /* Regular Lexer */
                                /*****************/

                                // Skip whitespace
                                if vec![' ', '\t'].contains(&letter) {
                                    word = self.add_word(word);
                                }
                                // Handle special symbols
                                else if self.symbols.contains(&letter) || letter == '\n' {
                                    word = self.pattern_add_symbol(word, letter);
                                }
                                // Handle word
                                else {
                                    self.word_push(&mut word, letter);
                                }
                            }
                        }
                    }
                }
            }
        }
        self.add_word(word);
        // If some region exists that was not closed
        if let Err((pos, region)) = self.region.is_region_closed(&self.reader) {
            return Err((
                LexerErrorType::Unclosed,
                PositionInfo::at_pos(self.path.clone(), pos, 0).data(region.name)
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::compiling_rules::{ Region, Rules };
    use crate::reg;
    use crate::compiling::{ Compiler, ScopingMode };

    #[test]
    fn test_lexer_base() {
        let symbols = vec!['(', ')'];
        let regions = reg![
            reg!(string as "String literal" => {
                begin: "'",
                end: "'"
            } => [
                reg!(array as "Array Literal" => {
                    begin: "[",
                    end: "]"
                })
            ])
        ];
        let expected = vec![
            ("let".to_string(), 1, 1),
            ("a".to_string(), 1, 5),
            ("=".to_string(), 1, 7),
            ("(".to_string(), 1, 9),
            ("12".to_string(), 1, 10),
            ("+".to_string(), 1, 13),
            ("32".to_string(), 1, 15),
            (")".to_string(), 1, 17)
        ];
        let rules = Rules::new(symbols, vec![], regions);
        let mut cc: Compiler = Compiler::new("TestScript", rules);
        cc.load("let a = (12 + 32)");
        let mut lexer = super::Lexer::new(&cc);
        let mut result = vec![];
        // Simulate lexing
        let res = lexer.run();
        assert!(res.is_ok());
        for lex in lexer.lexem {
            result.push((lex.word, lex.pos.0, lex.pos.1));
        }
        assert_eq!(expected, result);
    }

    #[test]
    fn test_lexer_string_interp() {
        let symbols = vec!['(', ')'];
        let regions = reg![
            reg!(string_literal as "String literal" => {
                begin: "'",
                end: "'"
            } => [
                reg!(string_interp as "String interpolation" => {
                    begin: "{",
                    end: "}",
                    tokenize: true
                } ref global)
            ])
        ];
        let expected = vec![
            ("let".to_string(), 1, 1),
            ("a".to_string(), 1, 5),
            ("=".to_string(), 1, 7),
            ("'this ".to_string(), 1, 9),
            ("{".to_string(), 1, 15),
            ("'is ".to_string(), 1, 16),
            ("{".to_string(), 1, 20),
            ("adjective".to_string(), 1, 21),
            ("}".to_string(), 1, 30),
            (" long'".to_string(), 1, 31),
            ("}".to_string(), 1, 37),
            (" 🎉 text'".to_string(), 1, 38)
        ];
        let rules = Rules::new(symbols, vec![], regions);
        let mut cc: Compiler = Compiler::new("TestScript", rules);
        cc.load("let a = 'this {'is {adjective} long'} 🎉 text'");
        let mut lexer = super::Lexer::new(&cc);
        let mut result = vec![];
        // Simulate lexing
        let res = lexer.run();
        assert!(res.is_ok());
        for lex in lexer.lexem {
            result.push((lex.word, lex.pos.0, lex.pos.1));
        }
        assert_eq!(expected, result);
    }

    #[test]
    fn test_lexer_indent_scoping_mode() {
        let symbols = vec![':'];
        let regions = reg![];
        let expected = vec![
            ("if".to_string(), (1, 1), 0),
            ("condition".to_string(), (1, 4), 3),
            (":".to_string(), (1, 13), 12),
            ("\n    ".to_string(), (2, 1), 13),
            ("if".to_string(), (2, 5), 18),
            ("subcondition".to_string(), (2, 8), 21),
            (":".to_string(), (2, 20), 33),
            ("\n        ".to_string(), (3, 1), 34),
            ("pass".to_string(), (3, 9), 43)
        ];
        let rules = Rules::new(symbols, vec![], regions);
        let mut cc: Compiler = Compiler::new("Testhon", rules);
        cc.scoping_mode = ScopingMode::Indent;
        cc.load(vec![
            "if condition:",
            "    if subcondition:",
            "        pass"
        ].join("\n"));
        let mut lexer = super::Lexer::new(&cc);
        let mut result = vec![];
        // Simulate lexing
        let res = lexer.run();
        assert!(res.is_ok());
        for lex in lexer.lexem {
            result.push((lex.word, (lex.pos.0, lex.pos.1), lex.start));
        }
        assert_eq!(expected, result);
    }

    #[test]
    fn test_lexer_manual_separator_mode() {
        let symbols = vec![';', '+', '='];
        let regions = reg![];
        let expected = vec![
            ("let".to_string(), 1, 1),
            ("age".to_string(), 1, 5),
            ("=".to_string(), 1, 9),
            ("12".to_string(), 1, 11),
            ("+".to_string(), 2, 1),
            ("12".to_string(), 3, 1),
            (";".to_string(), 3, 3)
        ];
        let rules = Rules::new(symbols, vec![], regions);
        let mut cc: Compiler = Compiler::new("Testhon", rules);
        cc.load(vec![
            "let age = 12",
            "+",
            "12;"
        ].join("\n"));
        let mut lexer = super::Lexer::new(&cc);
        let mut result = vec![];
        // Simulate lexing
        let res = lexer.run();
        assert!(res.is_ok());
        for lex in lexer.lexem {
            result.push((lex.word, lex.pos.0, lex.pos.1));
        }
        assert_eq!(expected, result);
    }

    #[test]
    fn test_lexer_multiline_regions() {
        let symbols = vec![';', '+', '='];
        let regions = reg![
            reg!(string as "String" => {
                begin: "'",
                end: "'"
            })
        ];
        let expected = vec![
            ("'this\nis\na\nmultiline\nstring'".to_string(), 1, 1)
        ];
        let rules = Rules::new(symbols, vec![], regions);
        let mut cc: Compiler = Compiler::new("Test", rules);
        cc.load(vec![
            "'this",
            "is",
            "a",
            "multiline",
            "string'",
        ].join("\n"));
        let mut lexer = super::Lexer::new(&cc);
        let mut result = vec![];
        // Simulate lexing
        let res = lexer.run();
        assert!(res.is_ok());
        for lex in lexer.lexem {
            result.push((lex.word, lex.pos.0, lex.pos.1));
        }
        assert_eq!(expected, result);
    }

    #[test]
    fn test_lexer_escaped_regions() {
        let symbols = vec![';', '+', '='];
        let regions = reg![
            reg!(string as "String" => {
                begin: "\"",
                end: "\""
            })
        ];
        let expected = vec![
            ("\"this is \\\"escaped\\\" string\"".to_string(), 1, 1)
        ];
        let rules = Rules::new(symbols, vec![], regions);
        let mut cc: Compiler = Compiler::new("Test", rules);
        cc.load(vec![
            "\"this is \\\"escaped\\\" string\""
        ].join("\n"));
        let mut lexer = super::Lexer::new(&cc);
        let mut result = vec![];
        // Simulate lexing
        let res = lexer.run();
        assert!(res.is_ok());
        for lex in lexer.lexem {
            result.push((lex.word, lex.pos.0, lex.pos.1));
        }
        assert_eq!(expected, result);
    }
}
