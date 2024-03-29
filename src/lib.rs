#![warn(missing_docs)]

//! # Heraclitus - the compiler frontend
//! 
//! With heraclitus you can create your language by skipping the cumbersome lexing step
//! and using convenience parsing methods that can get you started on your language much quicker.
//! 
//! The main construct that you need is the `Compiler`. The compiler will tokenize your code and assemble it
//! in a way that you can use to create AST by implementing predefined trait that helps you parse your code.
//! 
//! It's pretty simple. In order to get started you need 3 steps:
//! 1. Create lexing rules
//! 2. Create your ast nodes and let them implement trait provided by this package
//! 3. Create compiler and tie all the components together
//! 
//! Voilá!
//! Now you got yourself a ready to analyze / interpret / validate / compile AST.
//! 
//! Ready to get started?
//! # Example
//! ```
//! use heraclitus_compiler::prelude::*;
//! # let rules = Rules::new(vec![], vec![], reg![]);
//! Compiler::new("HerbScript", rules);
//! ```
//! It is recommended to use included prelude to import just the things we will actually need.
//! 
//! The `Compiler` requires lexer rules in order to exist.
//! 
//! ```
//! # use heraclitus_compiler::prelude::*;
//! # fn compiler() -> Result<(), LexerError> {
//! # let rules = Rules::new(vec![], vec![], reg![]);
//! let cc = Compiler::new("HerbScript", rules);
//! let tokens = cc.tokenize()?;
//! # Ok(())
//! # }
//! ```

pub mod compiling_rules;
pub mod compiling;

pub mod prelude {
    //! Use all the necessary modules
    //! 
    //! This package loads all the most necessary modules into the global scope.
    //! # Example
    //! ```
    //! use heraclitus_compiler::prelude::*;
    //! ```
    pub use crate::*;
    pub use crate::compiling_rules::*;
    pub use crate::compiling::*;
    pub use crate::compiling::patterns::*;
    pub use crate::compiling::failing::position_info::{PositionInfo, Position};
    pub use crate::compiling::failing::message::{Message, MessageType};
    pub use crate::compiling::failing::failure::Failure;
}