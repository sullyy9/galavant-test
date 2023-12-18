use chumsky::{prelude::*, text::Character};

use crate::{
    error::Error,
    expression::{Expr, ExprKind},
};

////////////////////////////////////////////////////////////////

pub fn parse_from_str(script: &str) -> Result<Vec<Expr>, Vec<Error>> {
    parser().parse(script)
}

////////////////////////////////////////////////////////////////

/// Parse a command and it's parameter.
///
/// # Arguments
///
/// * `cmd` - The command.
/// * `param_parser` - Parser for parsing the commands parameter.
///
/// # Returns
/// A parser that parses the command and it's parameter but only returns it's parameter in Box.
///   
fn command_with_param<'a, E>(
    cmd: &'a str,
    param_parser: E,
) -> impl Parser<char, Box<Expr>, Error = Error> + 'a
where
    E: Parser<char, Expr, Error = Error> + 'a,
{
    return text::keyword(cmd).ignore_then(param_parser).map(Box::from);
}

////////////////////////////////////////////////////////////////

/// Parse a command and it's 2 parameters.
///
/// # Arguments
///
/// * `cmd` - The command.
/// * `parser1` - Parser for parsing the commands first parameter.
/// * `parser2` - Parser for parsing the commands second parameter.
///
/// # Returns
/// A parser that parses the command and it's parameters but only returns it's parameters in Boxes
/// as a tuple.
///   
fn command_with_2_params<'a, E>(
    cmd: &'a str,
    parser1: E,
    parser2: E,
) -> impl Parser<char, (Box<Expr>, Box<Expr>), Error = Error> + 'a
where
    E: Parser<char, Expr, Error = Error> + 'a,
{
    return text::keyword(cmd)
        .ignore_then(parser1)
        .then(parser2)
        .map(|(p1, p2)| (Box::from(p1), Box::from(p2)));
}

////////////////////////////////////////////////////////////////

/// Parse a command and it's 5 parameters.
///
/// # Arguments
///
/// * `cmd` - The command.
/// * `parser` - Parser for parsing each of the commands parameters.
///
/// # Returns
/// A parser that parses the command and it's parameters but only returns it's parameters in Boxes
/// as a tuple.
///
/// # Todo
/// There's got to be a better way of doing this.
///   
fn command_with_5_params<'a, E>(
    cmd: &'a str,
    parser: E,
) -> impl Parser<char, (Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>), Error = Error> + 'a
where
    E: Parser<char, Expr, Error = Error> + Clone + 'a,
{
    return text::keyword(cmd)
        .ignore_then(parser.clone())
        .then(parser.clone())
        .then(parser.clone())
        .then(parser.clone())
        .then(parser)
        .map(|((((p1, p2), p3), p4), p5)| {
            (
                Box::from(p1),
                Box::from(p2),
                Box::from(p3),
                Box::from(p4),
                Box::from(p5),
            )
        });
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
fn command_with_params<'a, E>(
    cmd: &'a str,
    param_parser: E,
) -> impl Parser<char, Vec<Expr>, Error = Error> + 'a
where
    E: Parser<char, Vec<Expr>, Error = Error> + 'a,
{
    text::keyword(cmd).ignore_then(param_parser)
}

////////////////////////////////////////////////////////////////

fn parser() -> impl Parser<char, Vec<Expr>, Error = Error> {
    let whitespace = filter(|c: &char| c.is_inline_whitespace())
        .ignored()
        .repeated();

    // TODO: Allow escaped string delimeters within strings. (Don't think it was allowed on original
    // runtest but would be nice to have).
    let string = filter(|c| *c != '"')
        .repeated()
        .delimited_by(just('"'), just('"'))
        .map(String::from_iter)
        .map(ExprKind::String);

    let uint_dec = text::int(10).map(|s: String| ExprKind::UInt(s.parse().unwrap()));
    let uint_hex = just("$")
        .ignore_then(text::int(16))
        .map(|s: String| ExprKind::UInt(u32::from_str_radix(&s, 16).unwrap()));

    let uint = uint_dec.or(uint_hex);

    let expr = choice((string, uint))
        .map_with_span(Expr::from_kind_and_span)
        .padded_by(whitespace);
    let multi_expr = expr.separated_by(whitespace);

    ////////////////

    let hpmode = text::keyword("HPMODE").to(ExprKind::HPMode);
    let comment = command_with_param("COMMENT", expr).map(ExprKind::Comment);
    let wait = command_with_param("WAIT", expr).map(ExprKind::Wait);
    let opendialog = command_with_param("OPENDIALOG", expr).map(ExprKind::OpenDialog);
    let waitdialog = command_with_param("WAITDIALOG", expr).map(ExprKind::WaitDialog);
    let flush = text::keyword("FLUSH").to(ExprKind::Flush);
    let protocol = text::keyword("PROTOCOL").to(ExprKind::Protocol);
    let print = command_with_params("PRINT", multi_expr).map(ExprKind::Print);
    let settimeformat = command_with_param("SETTIMEFORMAT", expr).map(ExprKind::SetTimeFormat);
    let settime = text::keyword("SETTIME").to(ExprKind::SetTime);
    let setoption = command_with_2_params("SETOPTION", expr, expr)
        .map(|(option, setting)| ExprKind::SetOption { option, setting });

    let tcuclose = command_with_param("TCUCLOSE", expr).map(ExprKind::TCUClose);
    let tcuopen = command_with_param("TCUOPEN", expr).map(ExprKind::TCUOpen);
    let tcutest =
        command_with_5_params("TCUTEST", expr).map(|(channel, min, max, retries, message)| {
            ExprKind::TCUTest {
                channel,
                min,
                max,
                retries,
                message,
            }
        });

    let printerset = command_with_param("PRINTERSET", expr).map(ExprKind::PrinterSet);
    let printertest =
        command_with_5_params("PRINTERTEST", expr).map(|(channel, min, max, retries, message)| {
            ExprKind::PrinterTest {
                channel,
                min,
                max,
                retries,
                message,
            }
        });

    let usbopen = text::keyword("USBOPEN").to(ExprKind::USBOpen);
    let usbclose = text::keyword("USBCLOSE").to(ExprKind::USBClose);
    let usbprint = command_with_params("USBPRINT", multi_expr).map(ExprKind::USBPrint);
    let usbsettimeformat =
        command_with_param("USBSETTIMEFORMAT", expr).map(ExprKind::USBSetTimeFormat);
    let usbsettime = text::keyword("USBSETTIME").to(ExprKind::USBSetTime);
    let usbsetoption = command_with_2_params("USBSETOPTION", expr, expr)
        .map(|(option, setting)| ExprKind::USBSetOption { option, setting });

    let usbprinterset = command_with_param("USBPRINTERSET", expr).map(ExprKind::USBPrinterSet);
    let usbprintertest = command_with_5_params("USBPRINTERTEST", expr).map(
        |(channel, min, max, retries, message)| ExprKind::USBPrinterTest {
            channel,
            min,
            max,
            retries,
            message,
        },
    );

    let decl = choice((
        hpmode,
        comment,
        wait,
        opendialog,
        waitdialog,
        flush,
        protocol,
        print,
        settimeformat,
        settime,
        setoption,
        tcuclose,
        tcuopen,
        tcutest,
        printerset,
        printertest,
        usbopen,
        usbclose,
        usbprint,
        usbsettimeformat,
        usbsettime,
        usbsetoption,
        usbprinterset,
        usbprintertest,
    ))
    .map_with_span(Expr::from_kind_and_span)
    .padded_by(whitespace);

    ////////////////

    decl.or(expr)
        .separated_by(text::newline().repeated())
        .padded()
        .then_ignore(end())
}

////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::io::Write;

    use ariadne::{Label, Report, ReportKind, Source};

    use super::*;

    #[test]
    fn test_parse_commands() {
        let script = r#"
HPMODE
COMMENT "Test"
WAIT 1234
OPENDIALOG "Hello"
WAITDIALOG "PLEASE WAIT"
FLUSH
PROTOCOL
PRINT "print me"
SETTIMEFORMAT $A6
SETTIME
SETOPTION 4 6
TCUCLOSE 4
TCUOPEN $F
TCUTEST 5 12000 56000 0 "error"
PRINTERSET 1
PRINTERTEST 4 133 987 5 "error message"
USBOPEN
USBCLOSE
USBPRINT "Look at me I can print"
USBSETTIMEFORMAT 5
USBSETTIME
USBSETOPTION 5 9
USBPRINTERSET 6
USBPRINTERTEST 4 133 987 5 "error message"
        "#;

        let parsed_ast = parser().parse(script);

        let expected_ast = [
            ExprKind::HPMode,
            ExprKind::Comment(Box::new(Expr::from_str_default("Test"))),
            ExprKind::Wait(Box::new(Expr::from_uint_default(1234))),
            ExprKind::OpenDialog(Box::new(Expr::from_str_default("Hello"))),
            ExprKind::WaitDialog(Box::new(Expr::from_str_default("PLEASE WAIT"))),
            ExprKind::Flush,
            ExprKind::Protocol,
            ExprKind::Print(vec![Expr::from_str_default("print me")]),
            ExprKind::SetTimeFormat(Box::new(Expr::from_uint_default(0xA6))),
            ExprKind::SetTime,
            ExprKind::SetOption {
                option: Box::new(Expr::from_uint_default(4)),
                setting: Box::new(Expr::from_uint_default(6)),
            },
            ExprKind::TCUClose(Box::new(Expr::from_uint_default(4))),
            ExprKind::TCUOpen(Box::new(Expr::from_uint_default(0xF))),
            ExprKind::TCUTest {
                channel: Box::new(Expr::from_uint_default(5)),
                min: Box::new(Expr::from_uint_default(12000)),
                max: Box::new(Expr::from_uint_default(56000)),
                retries: Box::new(Expr::from_uint_default(0)),
                message: Box::new(Expr::from_str_default("error")),
            },
            ExprKind::PrinterSet(Box::new(Expr::from_uint_default(1))),
            ExprKind::PrinterTest {
                channel: Box::new(Expr::from_uint_default(4)),
                min: Box::new(Expr::from_uint_default(133)),
                max: Box::new(Expr::from_uint_default(987)),
                retries: Box::new(Expr::from_uint_default(5)),
                message: Box::new(Expr::from_str_default("error message")),
            },
            ExprKind::USBOpen,
            ExprKind::USBClose,
            ExprKind::USBPrint(vec![Expr::from_str_default("Look at me I can print")]),
            ExprKind::USBSetTimeFormat(Box::new(Expr::from_uint_default(5))),
            ExprKind::USBSetTime,
            ExprKind::USBSetOption {
                option: Box::new(Expr::from_uint_default(5)),
                setting: Box::new(Expr::from_uint_default(9)),
            },
            ExprKind::USBPrinterSet(Box::new(Expr::from_uint_default(6))),
            ExprKind::USBPrinterTest {
                channel: Box::new(Expr::from_uint_default(4)),
                min: Box::new(Expr::from_uint_default(133)),
                max: Box::new(Expr::from_uint_default(987)),
                retries: Box::new(Expr::from_uint_default(5)),
                message: Box::new(Expr::from_str_default("error message")),
            },
        ];
        let expected_ast = expected_ast.map(Expr::from_kind_default);

        match parsed_ast {
            Ok(ast) => {
                for (i, (actual_expr, expected_expr)) in
                    std::iter::zip(ast, expected_ast).enumerate()
                {
                    println!("{:?}", actual_expr);
                    assert_eq!(actual_expr, expected_expr, "At expression {}", i);
                }
            }
            Err(errors) => {
                if let Some(error) = errors.first() {
                    let report = {
                        let report = Report::build(ReportKind::Error, (), 0)
                            .with_label(Label::new(error.span()).with_message(error))
                            .finish();

                        let mut string = Vec::new();
                        report.write(Source::from(script), string.by_ref()).unwrap();

                        String::from_utf8(string)
                    };

                    if let Ok(report) = report {
                        panic!("Parser returned error: {}", report)
                    } else {
                        panic!("Parser returned error: {:#?}", report)
                    }
                }
            }
        }
    }

    #[test]
    fn test_single_command() {
        let script = r#"COMMENT "This is a comment 1234""#;

        match parse_from_str(script) {
            Ok(exprs) => {
                assert_eq!(exprs.len(), 1);
                assert_eq!(
                    exprs[0],
                    Expr::from_kind_default(ExprKind::Comment(Box::new(Expr::from_str_default(
                        "This is a comment 1234"
                    ))))
                )
            }
            Err(errors) => panic!("{:?}", errors),
        }
    }
}

////////////////////////////////////////////////////////////////
