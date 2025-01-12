use std::fmt::Debug;
use std::process::exit;

use crate::errors::SyntaxError;
use crate::lexer::tokens::{Kind, Token};
use crate::lexer::Lexer;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary {
        op: Kind,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Literal(Literal),
    FnCall {
        ident: String,
        params: Option<Vec<Expr>>,
    },
    Fn {
        ident: String,
        params: Option<Vec<String>>,
        operation: Box<Expr>,
    },
    Var(String),
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        alter: Box<Expr>,
    },
    Use(String),
}

#[derive(Clone, PartialEq)]
pub enum Literal {
    Str(String),
    Int(i32),
}

#[derive(PartialEq)]
pub struct Program(pub Vec<Expr>);

pub struct Parser {
    next: Option<Token>,
    lexer: Lexer,
}

impl Parser {
    pub fn new(program: String) -> Self {
        let mut lexer = Lexer::new(program);
        Self {
            next: lexer.get_next(),
            lexer,
        }
    }

    pub fn program(mut self) -> Program {
        if self.next.is_none() {
            println!("Program Terminated : Lookahead is empty, nothing to parse.");
            exit(1)
        }
        self.parse()
    }

    pub fn parse(&mut self) -> Program {
        let mut program: Vec<Expr> = vec![];

        while !self.next.is_none() {
            let exp = self.expr_def();
            program.push(match exp {
                Ok(val) => val,
                Err(e) => {
                    println!("{}", e);
                    exit(1)
                }
            });
        }
        Program(program)
    }

    fn expr_def(&mut self) -> Result<Expr, SyntaxError> {
        let expr = self.expr();
        self.eat(&mut Kind::SemiColon)?;
        expr
    }

    fn expr(&mut self) -> Result<Expr, SyntaxError> {
        let next_token = match &self.next {
            Some(token) => token,
            None => {
                return Err(SyntaxError(
                    vec![self.next_token().kind],
                    None,
                    (self.lexer.col_cursor, self.lexer.line_cursor),
                ))
            }
        };
        let expr = match next_token.kind {
            Kind::Fn => self.fun_expr(),
            Kind::Use => self.use_expr(),
            _ => Err(SyntaxError(
                vec![Kind::Fn],
                Some(next_token.kind.to_owned()),
                (self.lexer.col_cursor, self.lexer.line_cursor),
            )),
        };
        expr
    }

    fn use_expr(&mut self) -> Result<Expr, SyntaxError> {
        self.eat(&Kind::Use)?;
        let path = self.eat(&Kind::String)?.value;
        Ok(Expr::Use(path))
    }

    // ? No more function nesting (we call if_exprs and not expr everywhere)
    fn fun_expr(&mut self) -> Result<Expr, SyntaxError> {
        self.eat(&mut Kind::Fn)?;
        let id = self.eat(&Kind::Ident)?.value;

        // Check if the function has parameters (if it has the :: operator, it has parameters).
        if self.next_token().kind == Kind::DoubleColon {
            let mut params: Vec<String> = vec![];
            self.eat(&mut Kind::DoubleColon)?;
            while self.next_token().kind != Kind::Arrow {
                params.push(self.eat(&Kind::Ident)?.value);
                if self.next_token().kind == Kind::Comma {
                    self.eat(&Kind::Comma)?;
                }
            }

            self.eat(&Kind::Arrow)?;
            return Ok(Expr::Fn {
                ident: id,
                params: Some(params),
                operation: Box::new(self.if_expr()?),
            });
        }

        // If the function has no parameters, return a Expr::Fn with `None` as params value.
        self.eat(&Kind::Arrow)?;

        Ok(Expr::Fn {
            ident: id,
            params: None,
            operation: Box::new(self.if_expr()?),
        })
    }

    fn if_expr(&mut self) -> Result<Expr, SyntaxError> {
        if self.next_token().kind == Kind::If {
            self.eat(&Kind::If)?;
            let mut cond: Expr = Expr::Literal(Literal::Int(0));
            let mut then: Expr = Expr::Literal(Literal::Int(0));
            let mut alter: Expr = Expr::Literal(Literal::Int(0));
            while self.next_token().kind != Kind::QMark {
                cond = self.binary_expr()?
            }
            self.eat(&Kind::QMark)?;
            while self.next_token().kind != Kind::Colon {
                then = self.binary_expr()?
            }
            self.eat(&Kind::Colon)?;
            while self.next_token().kind != Kind::SemiColon {
                alter = self.binary_expr()?
            }
            return Ok(Expr::If {
                cond: Box::from(cond),
                then: Box::from(then),
                alter: Box::from(alter),
            });
        }
        self.binary_expr()
    }

    fn binary_expr(&mut self) -> Result<Expr, SyntaxError> {
        self.conditional_expr()
    }

    fn parenthesized_expr(&mut self) -> Result<Expr, SyntaxError> {
        self.eat(&Kind::LParen)?;
        let expr = self.binary_expr();
        self.eat(&Kind::RParen)?;
        expr
    }

    fn conditional_expr(&mut self) -> Result<Expr, SyntaxError> {
        let mut left: Expr = self.low_prec_expr()?;
        while self.next_token().get_prec() == 1 {
            let op = self.eat(&self.next_token().clone().kind)?.kind;
            let right = self.low_prec_expr()?;
            left = Expr::Binary {
                op,
                lhs: Box::new(left),
                rhs: Box::new(right),
            };
        }
        Ok(left)
    }

    // Operation such as +, - (expressions)
    fn low_prec_expr(&mut self) -> Result<Expr, SyntaxError> {
        let mut left = self.high_prec_expr()?;
        while self.next_token().get_prec() == 2 {
            let op = match self.eat(&self.next_token().clone().kind) {
                Ok(val) => val.kind,
                Err(e) => return Err(e),
            };
            let right = self.high_prec_expr()?;
            left = Expr::Binary {
                op,
                lhs: Box::new(left),
                rhs: Box::new(right),
            };
        }
        Ok(left)
    }

    // Operation such as *, /
    fn high_prec_expr(&mut self) -> Result<Expr, SyntaxError> {
        let mut left: Expr = self.factor()?;
        while self.next_token().get_prec() == 3 {
            let op = self.eat(&self.next_token().clone().kind)?.kind;
            let right = self.factor()?;
            left = Expr::Binary {
                op,
                lhs: Box::new(left),
                rhs: Box::new(right),
            };
        }
        Ok(left)
    }

    fn factor(&mut self) -> Result<Expr, SyntaxError> {
        let literal: Result<Expr, SyntaxError> = match self.next_token().kind {
            Kind::Integer => Ok(Expr::Literal(Literal::Int(
                match self.eat(&Kind::Integer) {
                    Ok(val) => val.value.to_string().parse::<i32>().unwrap(),
                    Err(e) => return Err(e),
                },
            ))),
            Kind::String => Ok(Expr::Literal(Literal::Str(self.eat(&Kind::String)?.value))),
            Kind::LParen => self.parenthesized_expr(),
            _ => self.ident(),
        };
        literal
    }

    fn ident(&mut self) -> Result<Expr, SyntaxError> {
        let id = self.eat(&Kind::Ident)?;
        if self.next_token().kind == Kind::LParen {
            let mut _params: Vec<Expr> = Vec::new();
            self.eat(&Kind::LParen)?;

            while self.next_token().kind != Kind::RParen {
                let param = self.conditional_expr()?;
                _params.push(param);
            }
            self.eat(&Kind::RParen)?;
            let params: Option<Vec<Expr>> = match _params.is_empty() {
                true => None,
                false => Some(_params),
            };
            return Ok(Expr::FnCall {
                ident: id.value,
                params,
            });
        }
        Ok(Expr::Var(id.value))
    }

    fn next_token(&self) -> &Token {
        self.next.as_ref().unwrap()
    }

    fn eat(&mut self, kind_target: &Kind) -> Result<Token, SyntaxError> {
        let t: Token = match &self.next {
            Some(val) => val.to_owned(),
            None => {
                return Err(SyntaxError(
                    vec![*kind_target],
                    None,
                    (self.lexer.col_cursor, self.lexer.line_cursor),
                ))
            }
        };

        if &t.kind != kind_target {
            return Err(SyntaxError(
                vec![*kind_target],
                Some(t.kind),
                (self.lexer.col_cursor, self.lexer.line_cursor),
            ));
        }

        self.next = self.lexer.get_next();

        Ok(t)
    }
}

impl Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Str(string) => write!(f, "{}", string),
            Literal::Int(int) => write!(f, "{}", int),
        }
    }
}

impl Debug for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().fold(Ok(()), |res, expr| {
            res.and_then(|_| writeln!(f, "{:#?}", expr))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fib_func() {
        assert_eq!(
            Parser::new(r#"fn fib :: n -> if n <= 1 ? n : fib(n - 1) + fib(n - 2);"#.to_owned())
                .program(),
            Program(vec![Expr::Fn {
                ident: "fib".to_owned(),
                params: Some(vec!["n".to_owned()]),
                operation: Box::from(Expr::If {
                    cond: Box::from(Expr::Binary {
                        op: Kind::Leq,
                        lhs: Box::from(Expr::Var("n".to_owned())),
                        rhs: Box::from(Expr::Literal(Literal::Int(1)))
                    }),
                    then: Box::from(Expr::Var("n".to_owned())),
                    alter: Box::from(Expr::Binary {
                        op: Kind::Plus,
                        lhs: Box::from(Expr::FnCall {
                            ident: "fib".to_owned(),
                            params: Some(vec![Expr::Binary {
                                op: Kind::Min,
                                lhs: Box::from(Expr::Var("n".to_owned())),
                                rhs: Box::from(Expr::Literal(Literal::Int(1)))
                            }])
                        }),
                        rhs: Box::from(Expr::FnCall {
                            ident: "fib".to_owned(),
                            params: Some(vec![Expr::Binary {
                                op: Kind::Min,
                                lhs: Box::from(Expr::Var("n".to_owned())),
                                rhs: Box::from(Expr::Literal(Literal::Int(2)))
                            }])
                        }),
                    })
                })
            }])
        );
    }
}
