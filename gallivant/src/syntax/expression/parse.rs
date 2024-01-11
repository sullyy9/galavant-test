use chumsky::{combinator::Repeated, prelude::*, text::Character};

use crate::syntax::error::Error;

use super::expression::ParsedExpr;

////////////////////////////////////////////////////////////////

/// Parser that matches inline whitepsace only. i.e. Whitespace not part of a newline. This differs
/// from chumsky's builtin whitespace parser which does match newline characters.
///
/// # Returns
/// A parser matching inline whitespace.
///   
pub fn whitespace() -> Repeated<impl Parser<char, (), Error = Error> + Copy + Clone> {
    filter(|c: &char| c.is_inline_whitespace())
        .ignored()
        .repeated()
}

////////////////////////////////////////////////////////////////

/// Parser that matches unsigned integers. This differs from chumsky's builtin text::int parser in
/// that it allows leading 0's.
///
/// # Arguments
/// * `radix` - Base of the integer.
///
/// # Returns
/// A parser matching unsigned integers.
///   
pub fn uint(radix: u32) -> impl Parser<char, String, Error = Error> + Copy + Clone {
    filter(move |c: &char| c.is_digit(radix))
        .map(Some)
        .chain::<char, Vec<_>, _>(filter(move |c: &char| c.is_digit(radix)).repeated())
        .collect()
}

////////////////////////////////////////////////////////////////

/// Takes a parser that outputs an expression and outputs a parser that outputs a comma seperated
/// list of those expressions.  
///
pub fn comma_seperated_list<P, const N: usize>(
    parsers: [P; N],
) -> impl Parser<char, [ParsedExpr; N], Error = Error>
where
    P: Parser<char, ParsedExpr, Error = Error> + 'static,
{
    let mut parsers = parsers.into_iter();
    let last = parsers.next_back();

    let mut output = empty().map(|_| Vec::new()).boxed();
    for parser in parsers {
        output = output
            .chain(parser.then_ignore(just(',').padded_by(whitespace())))
            .boxed();
    }

    if let Some(parser) = last {
        output = output.chain(parser).boxed();
    }

    output.map(|args| args.try_into().unwrap()).boxed()
}

////////////////////////////////////////////////////////////////

/// Parse a command and it's arguments.
///
/// # Arguments
///
/// * `cmd` - The command.
/// * `parsers` - Parser for parsing each argument in order.
///
/// # Returns
/// A parser that parses the command and it's arguments but only outputs it's arguments.
///   
pub fn command<P, const N: usize>(
    cmd: &'static str,
    parsers: [P; N],
) -> impl Parser<char, [Box<ParsedExpr>; N], Error = Error>
where
    P: Parser<char, ParsedExpr, Error = Error> + 'static,
{
    text::keyword(cmd)
        .then(whitespace())
        .ignore_then(comma_seperated_list(parsers).map(|args| args.map(|arg| Box::new(arg))))
        .boxed()
}

////////////////////////////////////////////////////////////////

/// Parse a command and it's parameters.
///
/// # Arguments
///
/// * `cmd` - The command.
/// * `expr` - Parser for parsing the commands parameters.
///
/// # Returns
/// A parser that parses the command and it's parameter but only returns it's parameters in a Vec.
///   
pub fn command_variadic<'a, E>(
    cmd: &'a str,
    param_parser: E,
) -> impl Parser<char, Vec<ParsedExpr>, Error = Error> + 'a
where
    E: Parser<char, ParsedExpr, Error = Error> + 'a,
{
    text::keyword(cmd)
        .then(whitespace())
        .ignore_then(param_parser.separated_by(just(',').padded_by(whitespace())))
}

////////////////////////////////////////////////////////////////
