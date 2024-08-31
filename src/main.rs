mod lexer;
mod parser;

use lexer::{Lexer, Token};
use parser::Parser;
use std::io::Read;

fn handle_definition<I>(p: &mut Parser<I>)
where
    I: Iterator<Item = char>,
{
    match p.parse_definition() {
        Ok(expr) => println!("parse 'def'\n{:?}", expr),
        Err(err) => {
            eprint!("error: {:?}", err);
            p.get_next_token();
        }
    }
}

fn handle_extern<I>(p: &mut Parser<I>)
where
    I: Iterator<Item = char>,
{
    match p.parse_extern() {
        Ok(expr) => println!("parse 'extern'\n{:?}", expr),
        Err(err) => {
            eprint!("error: {:?}", err);
            p.get_next_token();
        }
    }
}

fn handle_top_level_expression<I>(p: &mut Parser<I>)
where
    I: Iterator<Item = char>,
{
    match p.parse_top_level_expr() {
        Ok(expr) => println!("parse top-level expression\n{:?}", expr),
        Err(err) => {
            eprint!("error: {:?}", err);
            p.get_next_token();
        }
    }
}

fn main() {
    println!("Lex stdin");
    println!("ENTER to lex current input");
    println!("C-c   to exit");
    let lexer = Lexer::new(std::io::stdin().bytes().filter_map(|v| {
        let v = v.ok()?;
        Some(v.into())
    }));

    let mut parser = Parser::new(lexer);

    // throw first coin & init cur_token
    parser.get_next_token();

    loop {
        match *parser.cur_token() {
            Token::Eof => break,
            Token::Char(';') => {
                // ignore top level exp
                parser.get_next_token();
            }
            Token::Def => handle_definition(&mut parser),
            Token::Extern => handle_extern(&mut parser),
            _ => handle_top_level_expression(&mut parser),
        }
    }
}
