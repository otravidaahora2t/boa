//! Assignment operator parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Assignment
//! [spec]: https://tc39.es/ecma262/#sec-assignment-operators

mod arrow_function;
mod conditional;
mod exponentiation;

use self::{arrow_function::ArrowFunction, conditional::ConditionalExpression};
use crate::syntax::{
    ast::{node::Node, punc::Punctuator, token::TokenKind},
    parser::{AllowAwait, AllowIn, AllowYield, Cursor, ParseError, ParseResult, TokenParser},
};
pub(super) use exponentiation::ExponentiationExpression;

/// Assignment expression parsing.
///
/// This can be one of the following:
///
///  - [`ConditionalExpression`](../conditional_operator/struct.ConditionalExpression.html)
///  - `YieldExpression`
///  - [`ArrowFunction`](../../function/arrow_function/struct.ArrowFunction.html)
///  - `AsyncArrowFunction`
///  - [`LeftHandSideExpression`][lhs] `=` `AssignmentExpression`
///  - [`LeftHandSideExpression`][lhs] `AssignmentOperator` `AssignmentExpression`
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Assignment
/// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
/// [lhs]: ../lhs_expression/struct.LeftHandSideExpression.html
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct AssignmentExpression {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl AssignmentExpression {
    /// Creates a new `AssignmentExpression` parser.
    pub(in crate::syntax::parser) fn new<I, Y, A>(
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
    ) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl TokenParser for AssignmentExpression {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        // Arrow function
        let next_token = cursor.peek(0).ok_or(ParseError::AbruptEnd)?;
        match next_token.kind {
            // a=>{}
            TokenKind::Identifier(_)
                if cursor
                    .peek_expect_no_lineterminator(1, "arrow function")
                    .is_ok() =>
            {
                if let Some(tok) = cursor.peek(1) {
                    if tok.kind == TokenKind::Punctuator(Punctuator::Arrow) {
                        return ArrowFunction::new(
                            self.allow_in,
                            self.allow_yield,
                            self.allow_await,
                        )
                        .parse(cursor);
                    }
                }
            }
            // (a,b)=>{}
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                if let Some(node) =
                    ArrowFunction::new(self.allow_in, self.allow_yield, self.allow_await)
                        .try_parse(cursor)
                {
                    return Ok(node);
                }
            }
            _ => {}
        }

        let mut lhs = ConditionalExpression::new(self.allow_in, self.allow_yield, self.allow_await)
            .parse(cursor)?;
        // let mut lhs = self.read_block()?;

        if let Some(tok) = cursor.next() {
            match tok.kind {
                TokenKind::Punctuator(Punctuator::Assign) => {
                    lhs = Node::assign(lhs, self.parse(cursor)?)
                }
                TokenKind::Punctuator(p) if p.as_binop().is_some() => {
                    let expr = self.parse(cursor)?;
                    let binop = p.as_binop().expect("binop disappeared");
                    lhs = Node::bin_op(binop, lhs, expr);
                }
                _ => {
                    cursor.back();
                }
            }
        }

        Ok(lhs)
    }
}
