#[derive(Clone, PartialEq, Debug)]
/// A position in the Javascript source code
/// Stores both the column number and the line number
///
/// ## Similar Implementations
/// [V8: Location](https://cs.chromium.org/chromium/src/v8/src/parsing/scanner.h?type=cs&q=isValid+Location&g=0&l=216)
pub struct Position {
    // Column number
    pub column_number: u64,
    // Line number
    pub line_number: u64,
}

impl Position {
    /// Create a new position, positions are usually created by Tokens..
    ///
    /// # Arguments
    ///
    /// * `line_number` - The line number the token starts at
    /// * `column_number` - The column number the token starts at
    ///
    /// # Example from Token
    ///
    /// ```
    ///  pub fn new(data: TokenData, line_number: u64, column_number: u64) -> Token {
    ///     Token {
    ///         data: data,
    ///         pos: Position::new(line_number, column_number),
    ///     }
    /// }
    /// ```
    pub fn new(line_number: u64, column_number: u64) -> Position {
        Position {
            line_number: line_number,
            column_number: column_number,
        }
    }
}
