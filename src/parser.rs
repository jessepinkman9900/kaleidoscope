use crate::lexer::{Lexer, Token};

#[derive(Debug, PartialEq)]
pub enum ExpressionAST {
    // number - expression class for numeric literals
    Number(f64),

    // variable - expression class for referencing a variable
    Variable(String),

    // binary - expression class for binary operator
    Binary(char, Box<ExpressionAST>, Box<ExpressionAST>),

    // call - expression class for function calls
    Call(String, Vec<ExpressionAST>),
}

// PrototypeAST - represents the "prototype" for a function
// captures - names and argument names
#[derive(Debug, PartialEq)]
pub struct PrototypeAST(String, Vec<String>);

// FunctionAST - represent function definition
#[derive(Debug, PartialEq)]
pub struct FunctionAST(PrototypeAST, ExpressionAST);

// parse result - string as err type
type ParseResult<T> = Result<T, String>;

// parser
pub struct Parser<I>
where
    I: Iterator<Item = char>,
{
    lexer: Lexer<I>,
    cur_token: Option<Token>,
}

impl<I> Parser<I>
where
    I: Iterator<Item = char>,
{
    pub fn new(lexer: Lexer<I>) -> Self {
        Parser {
            lexer,
            cur_token: None,
        }
    }

    // --------------------
    // Simple Token Buffer
    // --------------------

    // impl global var `int CurToken`
    // panics if parser does NOT have cur token
    pub fn cur_token(&self) -> &Token {
        self.cur_token
            .as_ref()
            .expect("Parser: Expected cur_token!")
    }

    // advance `cur_token` by getting next token from lexer
    pub fn get_next_token(&mut self) {
        self.cur_token = Some(self.lexer.next_token());
    }

    // ------------------------
    // Basic Expression Parsing
    // ------------------------

    // number_expr := number
    fn parse_number_expr(&mut self) -> ParseResult<ExpressionAST> {
        match *self.cur_token() {
            Token::Number(number) => {
                // eat number token
                self.get_next_token();
                Ok(ExpressionAST::Number(number))
            }
            _ => unreachable!(),
        }
    }

    // paren_expr := '(' expression ')'
    fn parse_parenthesis_expr(&mut self) -> ParseResult<ExpressionAST> {
        // eat ( token
        assert_eq!(*self.cur_token(), Token::Char('('));
        self.get_next_token();

        let v = self.parse_expression()?;

        if *self.cur_token() == Token::Char(')') {
            // eat ) token
            self.get_next_token();
            Ok(v)
        } else {
            Err("expected ')'".into())
        }
    }

    // identifier_expr
    //      := identifier
    //      := identifier '(' expression* ')'
    fn parse_identifier_expr(&mut self) -> ParseResult<ExpressionAST> {
        let id_name = match self.cur_token.take() {
            Some(Token::Identifier(id)) => {
                // eat identifier token
                self.get_next_token();
                id
            }
            _ => unreachable!(),
        };

        if *self.cur_token() != Token::Char('(') {
            Ok(ExpressionAST::Variable(id_name))
        } else {
            // eat ( token
            self.get_next_token();
            let mut args: Vec<ExpressionAST> = Vec::new();

            // collect arguments
            if *self.cur_token() != Token::Char(')') {
                loop {
                    let arg = self.parse_expression()?;
                    args.push(arg);

                    if *self.cur_token() == Token::Char(')') {
                        // eat ) token
                        self.get_next_token();
                        break;
                    }

                    if *self.cur_token() != Token::Char(',') {
                        return Err("expected ')' or ',' in argument list".into());
                    }
                }

                self.get_next_token();
            }
            Ok(ExpressionAST::Call(id_name, args))
        }
    }

    // primary
    //      := identifier_expr
    //      := number_expr
    //      := paren_expr
    fn parse_primary(&mut self) -> ParseResult<ExpressionAST> {
        match *self.cur_token() {
            Token::Identifier(_) => self.parse_identifier_expr(),
            Token::Number(_) => self.parse_number_expr(),
            Token::Char('(') => self.parse_parenthesis_expr(),
            _ => Err("unkown token when expecting an expression".into()),
        }
    }

    // -------------------------
    // Binary Expression Parsing
    // -------------------------

    // expression
    //      := primary bin op rhs
    fn parse_expression(&mut self) -> ParseResult<ExpressionAST> {
        let lhs = self.parse_primary()?;
        self.parse_bin_op_rhs(0, lhs)
    }

    // bin op rhs
    //      := ('+' primar)*
    fn parse_bin_op_rhs(
        &mut self,
        expr_prec: isize,
        mut lhs: ExpressionAST,
    ) -> ParseResult<ExpressionAST> {
        loop {
            let token_prec = get_token_precedence(self.cur_token());

            // not a bin op or precendence too small
            if token_prec < expr_prec {
                return Ok(lhs);
            }

            let binop = match self.cur_token.take() {
                Some(Token::Char(c)) => {
                    // eat bin op token
                    self.get_next_token();
                    c
                }
                _ => unreachable!(),
            };

            // lhs BINOP1 rhs BINOP2 remrhs
            //     tok_prec   next_prec
            // parse primary expr after bin op
            let mut rhs = self.parse_primary()?;
            let next_prec = get_token_precedence(self.cur_token());
            if token_prec < next_prec {
                // binop2 has higher precendence than binop1, recurse into remrhs
                rhs = self.parse_bin_op_rhs(token_prec + 1, rhs)?
            }

            lhs = ExpressionAST::Binary(binop, Box::new(lhs), Box::new(rhs));
        }
    }

    // ----------------
    // Parsing the rest
    // ----------------
    fn parse_prototype(&mut self) -> ParseResult<PrototypeAST> {
        let id_name = match self.cur_token.take() {
            Some(Token::Identifier(id)) => {
                // eat identifier token
                self.get_next_token();
                id
            }
            other => {
                // plug back cur token
                self.cur_token = other;
                return Err("expected function name in prototype".into());
            }
        };

        if *self.cur_token() != Token::Char('(') {
            return Err("expected function name in prototype".into());
        }

        let mut args: Vec<String> = Vec::new();
        loop {
            self.get_next_token();
            match self.cur_token.take() {
                Some(Token::Identifier(arg)) => args.push(arg),
                Some(Token::Char(',')) => {}
                other => {
                    self.cur_token = other;
                    break;
                }
            }
        }

        if *self.cur_token() != Token::Char(')') {
            return Err("expected ')' in prototype".into());
        }
        // eat ) token
        self.get_next_token();

        Ok(PrototypeAST(id_name, args))
    }

    // definition := 'def' protype expression
    pub fn parse_definition(&mut self) -> ParseResult<FunctionAST> {
        // eat def token
        assert_eq!(*self.cur_token(), Token::Def);
        self.get_next_token();

        let proto = self.parse_prototype()?;
        let expr = self.parse_expression()?;

        Ok(FunctionAST(proto, expr))
    }

    // external := 'extern' prototype
    pub fn parse_extern(&mut self) -> ParseResult<PrototypeAST> {
        // eat extern token
        assert_eq!(*self.cur_token(), Token::Extern);
        self.get_next_token();

        self.parse_prototype()
    }

    // top_level_expr := expression
    pub fn parse_top_level_expr(&mut self) -> ParseResult<FunctionAST> {
        let e = self.parse_expression()?;
        let proto = PrototypeAST("".into(), Vec::new());
        Ok(FunctionAST(proto, e))
    }
}

// get the bin op precedence
fn get_token_precedence(tok: &Token) -> isize {
    match tok {
        Token::Char('<') => 10,
        Token::Char('+') => 20,
        Token::Char('-') => 20,
        Token::Char('*') => 40,
        _ => -1,
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    use super::{ExpressionAST, FunctionAST, Parser, PrototypeAST};
    use crate::lexer::Lexer;

    fn parser(input: &str) -> Parser<std::str::Chars> {
        let l = Lexer::new(input.chars());
        let mut p = Parser::new(l);

        // drop inital coin, init cur_tok
        p.get_next_token();

        p
    }

    #[test]
    fn parse_number() {
        let mut p = parser("13.37");

        assert_eq!(p.parse_number_expr(), Ok(ExpressionAST::Number(13.37f64)));
    }

    #[test]
    fn parse_variable() {
        let mut p = parser("foop");
        assert_eq!(
            p.parse_identifier_expr(),
            Ok(ExpressionAST::Variable("foop".into()))
        )
    }

    #[test]
    fn parse_primary() {
        let mut p = parser("1337 foop \n bla(123)");

        assert_eq!(p.parse_primary(), Ok(ExpressionAST::Number(1337f64)));
        assert_eq!(
            p.parse_identifier_expr(),
            Ok(ExpressionAST::Variable("foop".into()))
        );
        assert_eq!(
            p.parse_primary(),
            Ok(ExpressionAST::Call(
                "bla".into(),
                vec![ExpressionAST::Number(123f64)]
            ))
        );
    }

    #[test]
    fn parse_binary_op() {
        // operator before RHS has higher precendence
        //
        //       -
        //      / \
        //     +     c
        //    / \
        //   a   b
        let mut p = parser("a + b - c");

        let bin_expr_ab = ExpressionAST::Binary(
            '+',
            Box::new(ExpressionAST::Variable("a".into())),
            Box::new(ExpressionAST::Variable("b".into())),
        );

        let bin_expr_abc = ExpressionAST::Binary(
            '-',
            Box::new(bin_expr_ab),
            Box::new(ExpressionAST::Variable("c".into())),
        );

        assert_eq!(p.parse_expression(), Ok(bin_expr_abc));
    }

    #[test]
    fn parse_binary_op2() {
        // Operator after RHS has higher precedence, expected AST
        //
        //       +
        //      / \
        //     a   *
        //        / \
        //       b   c
        let mut p = parser("a + b * c");

        let bin_expr_bc = ExpressionAST::Binary(
            '*',
            Box::new(ExpressionAST::Variable("b".into())),
            Box::new(ExpressionAST::Variable("c".into())),
        );
        let bin_expr_abc = ExpressionAST::Binary(
            '+',
            Box::new(ExpressionAST::Variable("a".into())),
            Box::new(bin_expr_bc),
        );

        assert_eq!(p.parse_expression(), Ok(bin_expr_abc));
    }

    #[test]
    fn parse_prototype() {
        let mut p = parser("foo(a,b)");

        let proto = PrototypeAST("foo".into(), vec!["a".into(), "b".into()]);

        assert_eq!(p.parse_prototype(), Ok(proto));
    }

    #[test]
    fn parse_definition() {
        let mut p = parser("def bar( arg0, arg1) arg0 + arg1");

        let proto = PrototypeAST("bar".into(), vec!["arg0".into(), "arg1".into()]);
        let body = ExpressionAST::Binary(
            '+',
            Box::new(ExpressionAST::Variable("arg0".into())),
            Box::new(ExpressionAST::Variable("arg1".into())),
        );
        let func = FunctionAST(proto, body);

        assert_eq!(p.parse_definition(), Ok(func));
    }

    #[test]
    fn parse_extern() {
        let mut p = parser("extern bar()");

        let proto = PrototypeAST("bar".into(), vec![]);

        assert_eq!(p.parse_extern(), Ok(proto));
    }
}
