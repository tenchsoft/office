use super::{BinOp, Expr, FormulaError, UnaryOp};

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Text(String),
    Ident(String),
    CellRef { col: usize, row: usize },
    LParen,
    RParen,
    Comma,
    Colon,
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Amp,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Percent,
}

struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, FormulaError> {
        let mut tokens = Vec::new();
        while let Some(&ch) = self.chars.peek() {
            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    self.chars.next();
                }
                '(' => {
                    self.chars.next();
                    tokens.push(Token::LParen);
                }
                ')' => {
                    self.chars.next();
                    tokens.push(Token::RParen);
                }
                ',' => {
                    self.chars.next();
                    tokens.push(Token::Comma);
                }
                ':' => {
                    self.chars.next();
                    tokens.push(Token::Colon);
                }
                '+' => {
                    self.chars.next();
                    tokens.push(Token::Plus);
                }
                '-' => {
                    self.chars.next();
                    tokens.push(Token::Minus);
                }
                '*' => {
                    self.chars.next();
                    tokens.push(Token::Star);
                }
                '/' => {
                    self.chars.next();
                    tokens.push(Token::Slash);
                }
                '^' => {
                    self.chars.next();
                    tokens.push(Token::Caret);
                }
                '&' => {
                    self.chars.next();
                    tokens.push(Token::Amp);
                }
                '%' => {
                    self.chars.next();
                    tokens.push(Token::Percent);
                }
                '<' => {
                    self.chars.next();
                    if self.chars.peek() == Some(&'>') {
                        self.chars.next();
                        tokens.push(Token::Ne);
                    } else if self.chars.peek() == Some(&'=') {
                        self.chars.next();
                        tokens.push(Token::Le);
                    } else {
                        tokens.push(Token::Lt);
                    }
                }
                '>' => {
                    self.chars.next();
                    if self.chars.peek() == Some(&'=') {
                        self.chars.next();
                        tokens.push(Token::Ge);
                    } else {
                        tokens.push(Token::Gt);
                    }
                }
                '=' => {
                    self.chars.next();
                    tokens.push(Token::Eq);
                }
                '"' => {
                    self.chars.next();
                    let mut s = String::new();
                    while let Some(&c) = self.chars.peek() {
                        self.chars.next();
                        if c == '"' {
                            if self.chars.peek() == Some(&'"') {
                                self.chars.next();
                                s.push('"');
                            } else {
                                break;
                            }
                        } else {
                            s.push(c);
                        }
                    }
                    tokens.push(Token::Text(s));
                }
                _ if ch.is_ascii_digit() || ch == '.' => {
                    let mut num = String::new();
                    while let Some(&c) = self.chars.peek() {
                        if c.is_ascii_digit() || c == '.' {
                            num.push(c);
                            self.chars.next();
                        } else if (c == 'e' || c == 'E') && !num.contains('e') && !num.contains('E')
                        {
                            num.push(c);
                            self.chars.next();
                            // Consume optional sign after e/E
                            if let Some(&sign) = self.chars.peek() {
                                if sign == '+' || sign == '-' {
                                    num.push(sign);
                                    self.chars.next();
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    match num.parse::<f64>() {
                        Ok(n) => tokens.push(Token::Number(n)),
                        Err(_) => return Err(FormulaError::Value),
                    }
                }
                _ if ch.is_ascii_alphabetic() || ch == '_' => {
                    let mut ident = String::new();
                    while let Some(&c) = self.chars.peek() {
                        if c.is_ascii_alphanumeric() || c == '_' || c == '.' {
                            ident.push(c);
                            self.chars.next();
                        } else {
                            break;
                        }
                    }
                    // Check for cell reference like A1, AB123
                    if let Some((col, row)) = parse_cell_ref(&ident) {
                        tokens.push(Token::CellRef { col, row });
                    } else {
                        tokens.push(Token::Ident(ident.to_uppercase()));
                    }
                }
                _ => return Err(FormulaError::Name),
            }
        }
        Ok(tokens)
    }
}

/// Parse "A1" → (col=0, row=0), "B3" → (col=1, row=2), etc.
pub fn parse_cell_ref(s: &str) -> Option<(usize, usize)> {
    let s = s.trim();
    let mut col_str = String::new();
    let mut row_str = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphabetic() {
            if row_str.is_empty() {
                col_str.push(ch.to_ascii_uppercase());
            } else {
                return None; // letters after digits
            }
        } else if ch.is_ascii_digit() {
            row_str.push(ch);
        } else {
            return None;
        }
    }
    if col_str.is_empty() || row_str.is_empty() {
        return None;
    }
    let col = col_letters_to_index(&col_str)?;
    let row: usize = row_str.parse().ok()?;
    if row == 0 {
        return None;
    }
    Some((col, row - 1)) // 0-indexed
}

/// "A" → 0, "B" → 1, ..., "Z" → 25, "AA" → 26, etc.
pub fn col_letters_to_index(letters: &str) -> Option<usize> {
    let mut col = 0usize;
    for ch in letters.chars() {
        if !ch.is_ascii_alphabetic() {
            return None;
        }
        col = col * 26 + (ch as usize - 'A' as usize + 1);
    }
    Some(col - 1)
}

// ---------------------------------------------------------------------------
// Parser (recursive descent)
// ---------------------------------------------------------------------------

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), FormulaError> {
        match self.advance() {
            Some(t) if &t == expected => Ok(()),
            _ => Err(FormulaError::Value),
        }
    }

    /// Entry point: parse a full expression.
    pub fn parse(&mut self) -> Result<Expr, FormulaError> {
        let expr = self.parse_comparison()?;
        Ok(expr)
    }

    // comparison = concat (( "=" | "<>" | "<" | "<=" | ">" | ">=" ) concat)*
    fn parse_comparison(&mut self) -> Result<Expr, FormulaError> {
        let mut left = self.parse_concat()?;
        loop {
            let op = match self.peek() {
                Some(Token::Eq) => BinOp::Eq,
                Some(Token::Ne) => BinOp::Ne,
                Some(Token::Lt) => BinOp::Lt,
                Some(Token::Le) => BinOp::Le,
                Some(Token::Gt) => BinOp::Gt,
                Some(Token::Ge) => BinOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_concat()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    // concat = additive ( "&" additive )*
    fn parse_concat(&mut self) -> Result<Expr, FormulaError> {
        let mut left = self.parse_additive()?;
        while matches!(self.peek(), Some(Token::Amp)) {
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinOp {
                op: BinOp::Concat,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    // additive = multiplicative (("+" | "-") multiplicative)*
    fn parse_additive(&mut self) -> Result<Expr, FormulaError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    // multiplicative = power (("*" | "/") power)*
    fn parse_multiplicative(&mut self) -> Result<Expr, FormulaError> {
        let mut left = self.parse_power()?;
        loop {
            let op = match self.peek() {
                Some(Token::Star) => BinOp::Mul,
                Some(Token::Slash) => BinOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_power()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    // power = unary ("^" unary)*
    fn parse_power(&mut self) -> Result<Expr, FormulaError> {
        let mut left = self.parse_unary()?;
        while matches!(self.peek(), Some(Token::Caret)) {
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinOp {
                op: BinOp::Pow,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    // unary = ("-" | "+") unary | postfix
    fn parse_unary(&mut self) -> Result<Expr, FormulaError> {
        match self.peek() {
            Some(Token::Minus) => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                })
            }
            Some(Token::Plus) => {
                self.advance();
                self.parse_unary()
            }
            _ => self.parse_postfix(),
        }
    }

    // postfix = primary ( "%" )?
    fn parse_postfix(&mut self) -> Result<Expr, FormulaError> {
        let mut expr = self.parse_primary()?;
        if matches!(self.peek(), Some(Token::Percent)) {
            self.advance();
            expr = Expr::UnaryOp {
                op: UnaryOp::Percent,
                operand: Box::new(expr),
            };
        }
        Ok(expr)
    }

    // primary = Number | Text | Bool | CellRef | Function | "(" expr ")" | Range
    fn parse_primary(&mut self) -> Result<Expr, FormulaError> {
        match self.peek().cloned() {
            Some(Token::Number(n)) => {
                self.advance();
                Ok(Expr::Number(n))
            }
            Some(Token::Text(s)) => {
                self.advance();
                Ok(Expr::Text(s))
            }
            Some(Token::Ident(name)) => {
                self.advance();
                // Check for TRUE/FALSE
                if name == "TRUE" {
                    return Ok(Expr::Bool(true));
                }
                if name == "FALSE" {
                    return Ok(Expr::Bool(false));
                }
                // Must be a function call
                if matches!(self.peek(), Some(Token::LParen)) {
                    self.advance(); // consume '('
                    let args = self.parse_args()?;
                    self.expect(&Token::RParen)?;
                    Ok(Expr::Function { name, args })
                } else {
                    // Named reference
                    Ok(Expr::Name(name))
                }
            }
            Some(Token::CellRef { col, row }) => {
                self.advance();
                // Check for range: CellRef : CellRef
                if matches!(self.peek(), Some(Token::Colon)) {
                    self.advance();
                    if let Some(Token::CellRef {
                        col: end_col,
                        row: end_row,
                    }) = self.peek().cloned()
                    {
                        self.advance();
                        Ok(Expr::Range {
                            start_col: col,
                            start_row: row,
                            end_col,
                            end_row,
                        })
                    } else {
                        Err(FormulaError::Ref)
                    }
                } else {
                    Ok(Expr::CellRef { col, row })
                }
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_comparison()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            _ => Err(FormulaError::Value),
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, FormulaError> {
        let mut args = Vec::new();
        if matches!(self.peek(), Some(Token::RParen)) {
            return Ok(args);
        }
        args.push(self.parse_comparison()?);
        while matches!(self.peek(), Some(Token::Comma)) {
            self.advance();
            args.push(self.parse_comparison()?);
        }
        Ok(args)
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse a formula string (without the leading `=`) into an AST.
pub fn parse_formula(input: &str) -> Result<Expr, FormulaError> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}
