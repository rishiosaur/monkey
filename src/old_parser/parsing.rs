use crate::{
    lexer::{TokenType},
    ast::nodes::{
        Program,
        Statement,
        Expression
    }
};

use super::{
    parser::Parser,
    precedences::{int_precedence, Precedence},
};

impl<'a> Parser<'a> {
    pub fn parseProgram(&mut self) -> Program {
        let mut statements = Vec::<Box<&Statement>>::new();

        for tok in self.tokens.iter() {
            let stmt = self.parseStatement();

            statements.push(stmt);
        }

        Program {
            statements,
        }
    }

    fn parseStatement(&mut self) -> Box<&Statement> {
        match self.current_token().unwrap().kind {
            crate::lexer::TokenType::Value => self.parseValueStatement(),
            crate::lexer::TokenType::Update => self.parseUpdateStatement(),
            crate::lexer::TokenType::Return => self.parseReturnStatement(),
            _ => self.parseExpressionStatement(),
        }
    }

    fn parseReturnStatement(&mut self) -> Box<&Statement> {
        let token = self.current_token().unwrap();
        self.consume_token();
        let value = self.parseExpression(Precedence::LOWEST);

        while !self.current_token_is(TokenType::Semicolon) {
            self.consume_token();
        }

        Box::new(&Statement::ReturnStatement {
            token,
            value,
        })
    }

    fn parseValueStatement(&mut self) -> Box<dyn Statement> {
        let tok = self.current_token().unwrap();

        self.expect_peek(TokenType::Identifier(String::new()));

        let name = IdentifierLiteral {
            token: self.current_token().unwrap(),
        };

        self.expect_peek(TokenType::Assign);

        self.consume_token();

        let value = self.parseExpression(Precedence::LOWEST);

        if self.peek_token_is(TokenType::Semicolon) {
            self.consume_token();
        }

        Box::new(ValueStatement {
            token: tok,
            name: name,
            value: value,
        })
    }

    fn parseUpdateStatement(&mut self) -> Box<dyn Statement> {
        let tok = self.current_token().unwrap();

        self.expect_peek(TokenType::Identifier(String::new()));

        let name = IdentifierLiteral {
            token: self.current_token().unwrap(),
        };

        self.expect_peek(TokenType::Assign);

        self.consume_token();

        let value = self.parseExpression(Precedence::LOWEST);

        if self.peek_token_is(TokenType::Semicolon) {
            self.consume_token();
        }

        Box::new(UpdateStatement {
            token: tok,
            name: name,
            value: value,
        })
    }

    fn parseExpressionStatement(&mut self) -> Box<ExpressionStatement> {
        let expression = self.parseExpression(Precedence::LOWEST);
        let tok= self.current_token().unwrap();

        if self.current_token_is(TokenType::Semicolon) {
            self.consume_token();
        }

        Box::new(ExpressionStatement {
            expression: expression,
            token: tok,
        })
    }

    fn parseExpression(&mut self, precedence: Precedence) -> Box<&'a Expression<'a>> {
        let leftExp = self.matchPrefixExpression(self.current_token().unwrap().kind);

        while !self.peek_token_is(TokenType::Semicolon)
            && int_precedence(precedence) < int_precedence(self.peek_precedence())
        {
            let infixed = self.matchInfixExpression(self.current_token().unwrap().kind, leftExp.clone());

            match infixed {
                None => return leftExp,
                Some(infix) => {
                    self.consume_token();
                    return infix;
                }
            }
        }

        return leftExp;
    }

    fn matchPrefixExpression(&mut self, kind: TokenType) -> Box<Expression> {
        match kind {
            TokenType::Integer(_) => self.parseIntegerExpression(),
            TokenType::String(_) => self.parseStringExpression(),
            TokenType::Boolean(_) => self.parseBooleanExpression(),
            TokenType::Identifier(_) => self.parseIdentifierExpression(),
            TokenType::Minus => self.parsePrefixExpression(),
            _ => panic!(
                "[{}] PARSER ERROR: NO PREFIX FUNCTION FOUND FOR {:?}",
                self.position, kind
            ),
        }
    }

    fn parseBooleanExpression(&self) -> Box<dyn Expression> {
        Box::new(BooleanLiteral {
            token: self.current_token().unwrap(),
        })
    }

    fn parseIntegerExpression(&self) -> Box<dyn Expression> {
        Box::new(IntegerLiteral {
            token: self.current_token().unwrap(),
        })
    }

    fn parseStringExpression(&self) -> Box<dyn Expression> {
        Box::new(StringLiteral {
            token: self.current_token().unwrap(),
        })
    }

    fn parseIdentifierExpression(&self) -> Box<&'a Expression> {
        Box::new(IdentifierLiteral {
            token: self.current_token().unwrap(),
        })
    }

    fn parsePrefixExpression(&mut self) -> Box<dyn Expression> {
        let tok = self.current_token().unwrap();

        self.consume_token();

        let right = self.parseExpression(Precedence::LOWEST);

        Box::new(PrefixExpression {
            token: tok,
            
            right: right,
        })
    }

    fn matchInfixExpression(
        &mut self,
        kind: TokenType,
        leftExpression: Box<dyn Expression>,
    ) -> Option<Box<dyn Expression>> {
        match kind {
            TokenType::Plus => Some(self.parseInfixExpression(leftExpression)),
            TokenType::Minus => Some(self.parseInfixExpression(leftExpression)),
            TokenType::Asterisk => Some(self.parseInfixExpression(leftExpression)),
            TokenType::Slash => Some(self.parseInfixExpression(leftExpression)),
            TokenType::Equal => Some(self.parseInfixExpression(leftExpression)),
            TokenType::NotEqual => Some(self.parseInfixExpression(leftExpression)),
            TokenType::LT => Some(self.parseInfixExpression(leftExpression)),
            TokenType::GT => Some(self.parseInfixExpression(leftExpression)),
            TokenType::LTEq => Some(self.parseInfixExpression(leftExpression)),
            TokenType::GTEq => Some(self.parseInfixExpression(leftExpression)),
            _ => None,
        }
    }

    fn parseInfixExpression(&mut self, left: Box<dyn Expression>) -> Box<dyn Expression> {
        let tok = self.current_token().unwrap();
        

        self.consume_token();

        let right = self.parseExpression(Precedence::LOWEST);

        Box::new(InfixExpression {
            token: tok,
            
            right: right,
            left: left,
        })
    }
}
