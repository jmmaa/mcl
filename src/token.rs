#[derive(Debug)]
pub enum LiteralKind<'a> {
    String(Token<'a>),
    Number(Token<'a>),
    True,
    False,
    Null,
}

#[derive(Debug)]
pub enum IdentifierKind<'a> {
    String(Token<'a>),
}

#[derive(Debug)]
pub enum DelimiterKind {
    TableTerm,
    TablePrec,
    ListPrec,
    ListTerm,
}

#[derive(Debug)]
pub enum TokenKind<'a> {
    Identifier(IdentifierKind<'a>),
    Literal(LiteralKind<'a>),
    Delimiter(DelimiterKind),
}

#[derive(Debug)]
pub struct Position {
    l: usize,
    c: usize,
    i: usize,
}

impl<'a> Position {
    #[inline(always)]
    pub fn new(l: usize, c: usize, i: usize) -> Position {
        Position { l, c, i }
    }

    #[inline(always)]
    pub fn line(&'a self) -> usize {
        self.l
    }

    #[inline(always)]
    pub fn column(&'a self) -> usize {
        self.c
    }

    #[inline(always)]
    pub fn index(&'a self) -> usize {
        self.i
    }
}

#[derive(Debug)]
pub struct Location {
    s: Position,
    e: Position,
}

impl<'a> Location {
    #[inline(always)]
    pub fn new(s: Position, e: Position) -> Location {
        Location { s, e }
    }

    #[inline(always)]
    pub fn start(&'a self) -> &'a Position {
        &self.s
    }

    #[inline(always)]
    pub fn end(&'a self) -> &'a Position {
        &self.e
    }
}

#[derive(Debug)]
pub struct Token<'a> {
    l: Location,
    b: &'a [u8],
}

impl<'a> Token<'a> {
    #[inline(always)]
    pub fn new(l: Location, b: &'a [u8]) -> Token<'a> {
        Token { l, b }
    }

    #[inline(always)]
    pub fn bytes(&'a self) -> &'a [u8] {
        self.b
    }

    #[inline(always)]
    pub fn loc(&'a self) -> &'a Location {
        &self.l
    }
}
