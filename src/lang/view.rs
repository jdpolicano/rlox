use std::fmt;

/// The logical "view" into a buffer from the perspective of a text editor. It is more or less how a user would expect to find a
/// given substring inside a document.
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct View {
    pub line: usize,   // logical line number
    pub column: usize, // logical column number
}

impl View {
    pub fn new(line: usize, column: usize) -> View {
        View { line, column }
    }

    pub fn inc_line(&mut self) {
        self.line += 1;
        self.column = 0;
    }

    pub fn inc_col(&mut self) {
        self.column += 1;
    }
}

impl Default for View {
    fn default() -> Self {
        View::new(0, 0)
    }
}

impl fmt::Display for View {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "@({}:{})", self.line, self.column)
    }
}
