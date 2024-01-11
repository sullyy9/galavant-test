use chumsky::{prelude::*, text::newline};

use crate::syntax::error::{Error, ErrorNote};

use super::{
    expression::{Expr, ParsedExpr},
    parse,
};

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExprKind {
    String,
    UInt,

    ScriptComment,

    HPMode,
    Comment,
    Wait,
    OpenDialog,
    WaitDialog,
    Flush,
    Protocol,
    Print,
    SetTimeFormat,
    SetTime,
    SetOption,
    TCUClose,
    TCUOpen,
    TCUTest,
    PrinterSet,
    PrinterTest,
    IssueTest,
    TestResult,
    USBOpen,
    USBClose,
    USBPrint,
    USBSetTimeFormat,
    USBSetTime,
    USBSetOption,
    USBPrinterSet,
    USBPrinterTest,
}

////////////////////////////////////////////////////////////////

impl ExprKind {
    pub fn name(&self) -> &'static str {
        match self {
            ExprKind::String => "String",
            ExprKind::UInt => "Unsigned Integer",

            ExprKind::ScriptComment => "Script Comment",

            ExprKind::HPMode => "Command: 'HPMODE'",
            ExprKind::Comment => "Command: 'COMMENT'",
            ExprKind::Wait => "Command: 'WAIT'",
            ExprKind::OpenDialog => "Command: 'OPENDIALOG'",
            ExprKind::WaitDialog => "Command: 'WAITDIALOG'",
            ExprKind::Flush => "Command: 'FLUSH'",
            ExprKind::Protocol => "Command: 'PROTOCOL'",
            ExprKind::Print => "Command: 'PRINT'",
            ExprKind::SetTimeFormat => "Command: 'SETTIMEFORMAT'",
            ExprKind::SetTime => "Command: 'SETTIME'",
            ExprKind::SetOption => "Command: 'SETOPTION'",
            ExprKind::TCUClose => "Command: 'TCUCLOSE'",
            ExprKind::TCUOpen => "Command: 'TCUOPEN'",
            ExprKind::TCUTest => "Command: 'TCUTEST'",
            ExprKind::PrinterSet => "Command: 'PRINTERSET'",
            ExprKind::PrinterTest => "Command: 'PRINTERTEST'",
            ExprKind::IssueTest => "Command: 'ISSUETEST'",
            ExprKind::TestResult => "Command: 'TESTRESULT'",
            ExprKind::USBOpen => "Command: 'USBOPEN'",
            ExprKind::USBClose => "Command: 'USBCLOSE'",
            ExprKind::USBPrint => "Command: 'USBPRINT'",
            ExprKind::USBSetTimeFormat => "Command: 'USBSETTIMEFORMAT'",
            ExprKind::USBSetTime => "Command: 'USBSETTIME'",
            ExprKind::USBSetOption => "Command: 'USBSETOPTION'",
            ExprKind::USBPrinterSet => "Command: 'USBPRINTERSET'",
            ExprKind::USBPrinterTest => "Command: 'USBPRINTERTEST'",
        }
    }

    /// Return a parser for a given kind of expression.
    ///
    pub fn parser(&self) -> impl Parser<char, ParsedExpr, Error = Error> + Clone {
        match self {
            ////////////////////////////////////////////////////////////////
            ExprKind::String => filter(|c| *c != '"')
                .repeated()
                .delimited_by(just('"'), just('"'))
                .map(String::from_iter)
                .map(Expr::String)
                .boxed(),

            ////////////////////////////////////////////////////////////////
            ExprKind::UInt => {
                let uint_dec = parse::uint(10).map(|s: String| Expr::UInt(s.parse().unwrap()));
                let uint_hex = just("$")
                    .ignore_then(parse::uint(16))
                    .map(|s: String| Expr::UInt(u32::from_str_radix(&s, 16).unwrap()));

                choice((uint_dec, uint_hex)).boxed()
            }

            ////////////////////////////////////////////////////////////////
            ExprKind::ScriptComment => just(';')
                .ignore_then(take_until(choice((newline(), end())).rewind()))
                .map(|(s, _)| String::from_iter(s))
                .map(Expr::ScriptComment)
                .padded_by(parse::whitespace())
                .boxed(),

            ////////////////////////////////////////////////////////////////
            ExprKind::HPMode => text::keyword("HPMODE").to(Expr::HPMode).boxed(),

            ExprKind::Comment => parse::command("COMMENT", [validate_string(argument())])
                .map(|[arg]| Expr::Comment(arg))
                .boxed(),

            ExprKind::Wait => parse::command("WAIT", [validate_uint(argument())])
                .map(|[arg]| Expr::Wait(arg))
                .boxed(),

            ExprKind::OpenDialog => parse::command("OPENDIALOG", [validate_string(argument())])
                .map(|[arg]| Expr::OpenDialog(arg))
                .boxed(),

            ExprKind::WaitDialog => parse::command("WAITDIALOG", [validate_string(argument())])
                .map(|[arg]| Expr::WaitDialog(arg))
                .boxed(),

            ExprKind::Flush => text::keyword("FLUSH").to(Expr::Flush).boxed(),

            ExprKind::Protocol => text::keyword("PROTOCOL").to(Expr::Protocol).boxed(),

            ExprKind::Print => parse::command_variadic("PRINT", argument())
                .map(Expr::Print)
                .boxed(),

            ExprKind::SetTimeFormat => parse::command("SETTIMEFORMAT", [validate_byte(argument())])
                .map(|[arg]| Expr::SetTimeFormat(arg))
                .boxed(),

            ExprKind::SetTime => text::keyword("SETTIME").to(Expr::SetTime).boxed(),

            ExprKind::SetOption => parse::command(
                "SETOPTION",
                [validate_byte(argument()), validate_byte(argument())],
            )
            .map(|[option, setting]| Expr::SetOption { option, setting })
            .boxed(),

            ExprKind::TCUClose => parse::command("TCUCLOSE", [validate_byte(argument())])
                .map(|[arg]| Expr::TCUClose(arg))
                .boxed(),

            ExprKind::TCUOpen => parse::command("TCUOPEN", [validate_byte(argument())])
                .map(|[arg]| Expr::TCUOpen(arg))
                .boxed(),

            ExprKind::TCUTest => parse::command(
                "TCUTEST",
                [
                    validate_byte(argument()),
                    validate_uint(argument()),
                    validate_uint(argument()),
                    validate_uint(argument()),
                    validate_string(argument()),
                ],
            )
            .map(|[channel, min, max, retries, message]| Expr::TCUTest {
                channel,
                min,
                max,
                retries,
                message,
            })
            .boxed(),

            ExprKind::PrinterSet => parse::command("PRINTERSET", [validate_byte(argument())])
                .map(|[arg]| Expr::PrinterSet(arg))
                .boxed(),

            ExprKind::PrinterTest => parse::command(
                "PRINTERTEST",
                [
                    validate_byte(argument()),
                    validate_uint(argument()),
                    validate_uint(argument()),
                    validate_uint(argument()),
                    validate_string(argument()),
                ],
            )
            .map(|[channel, min, max, retries, message]| Expr::PrinterTest {
                channel,
                min,
                max,
                retries,
                message,
            })
            .boxed(),

            ExprKind::IssueTest => todo!(),
            ExprKind::TestResult => todo!(),

            ExprKind::USBOpen => text::keyword("USBOPEN").to(Expr::USBOpen).boxed(),
            ExprKind::USBClose => text::keyword("USBCLOSE").to(Expr::USBClose).boxed(),

            ExprKind::USBPrint => parse::command_variadic("USBPRINT", argument())
                .map(Expr::USBPrint)
                .boxed(),

            ExprKind::USBSetTimeFormat => {
                parse::command("USBSETTIMEFORMAT", [validate_byte(argument())])
                    .map(|[arg]| Expr::USBSetTimeFormat(arg))
                    .boxed()
            }

            ExprKind::USBSetTime => text::keyword("USBSETTIME").to(Expr::USBSetTime).boxed(),

            ExprKind::USBSetOption => parse::command(
                "USBSETOPTION",
                [validate_byte(argument()), validate_byte(argument())],
            )
            .map(|[option, setting]| Expr::USBSetOption { option, setting })
            .boxed(),

            ExprKind::USBPrinterSet => parse::command("USBPRINTERSET", [validate_byte(argument())])
                .map(|[arg]| Expr::USBPrinterSet(arg))
                .boxed(),

            ExprKind::USBPrinterTest => parse::command(
                "USBPRINTERTEST",
                [
                    validate_byte(argument()),
                    validate_uint(argument()),
                    validate_uint(argument()),
                    validate_uint(argument()),
                    validate_string(argument()),
                ],
            )
            .map(
                |[channel, min, max, retries, message]| Expr::USBPrinterTest {
                    channel,
                    min,
                    max,
                    retries,
                    message,
                },
            )
            .boxed(),
        }
        .map_with_span(ParsedExpr::from_kind_and_span)
    }
}

////////////////////////////////////////////////////////////////

/// Parser that matches any value type. i.e. a String or UInt.
///
fn argument() -> impl Parser<char, ParsedExpr, Error = Error> + Clone {
    choice((ExprKind::String.parser(), ExprKind::UInt.parser())).padded_by(parse::whitespace())
}

////////////////////////////////////////////////////////////////

/// Takes a parser and validates that the output is a String. If not, it outputs an error.
///
fn validate_string<'a, 'b, P>(parser: P) -> BoxedParser<'b, char, ParsedExpr, Error>
where
    P: Parser<char, ParsedExpr, Error = Error> + 'a,
    'a: 'b,
{
    parser
        .validate(|arg, span, emit| {
            if !matches!(arg.expression(), Expr::String(_)) {
                emit(
                Error::argument_type(span, [ExprKind::String], arg.expression_kind())
                    .with_note(ErrorNote::Note(
                    "If the argument was intended to be a string it should be delimited by \"\"",
                )),
            )
            }

            arg
        })
        .boxed()
}

////////////////////////////////////////////////////////////////

/// Takes a parser and validates that the output is a Uint. If not, it outputs an error.
/// If it isn't a string, it outputs an error.
///
fn validate_uint<'a, 'b, P>(parser: P) -> BoxedParser<'b, char, ParsedExpr, Error>
where
    P: Parser<char, ParsedExpr, Error = Error> + 'a,
    'a: 'b,
{
    parser.validate(|arg, span, emit| {
        if !matches!(arg.expression(), Expr::UInt(_)) {
            let mut error = Error::argument_type(span, [ExprKind::UInt], arg.expression_kind());

            if let Expr::String(string) = arg.expression() {
                if string.chars().all(|c| c.is_numeric()) {
                    error = error.with_note(ErrorNote::Help("If the argument was intended to be an unsigned integer, try removing the enclosing \"\""));
                } else if string.starts_with('$') && string.chars().skip(1).all(|c| c.is_ascii_hexdigit()) {
                    error = error.with_note(ErrorNote::Help("If the argument was intended to be a hex unsigned integer, try removing the enclosing \"\""));
                }
            }
            emit(error)
        }
        arg
    }).boxed()
}

////////////////////////////////////////////////////////////////

/// Takes a parser and validates that the output is a UInt < 256. If not, it outputs an error.
/// If it isn't a string, it outputs an error.
///
fn validate_byte<'a, 'b, P>(parser: P) -> BoxedParser<'b, char, ParsedExpr, Error>
where
    P: Parser<char, ParsedExpr, Error = Error> + 'a,
    'a: 'b,
{
    parser
        .validate(|arg, span, emit| {
            if let Expr::UInt(value) = arg.expression() {
                if *value > 255 {
                    emit(Error::argument_value_size(span, *value, (0, 255)))
                }
            }

            arg
        })
        .boxed()
}

////////////////////////////////////////////////////////////////
