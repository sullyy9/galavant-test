use chumsky::{
    combinator::Repeated,
    prelude::*,
    text::{newline, Character},
};

use super::{
    error::{Error, ErrorReason},
    expression::{Expr, ExprKind, ParsedExpr},
};

////////////////////////////////////////////////////////////////

pub fn parse_from_str(script: &str) -> Result<Vec<ParsedExpr>, Vec<Error>> {
    parser().parse(script)
}

////////////////////////////////////////////////////////////////

/// Parser that matches inline whitepsace only. i.e. Whitespace not part of a newline. This differs
/// from chumsky's builtin whitespace parser which does match newline characters.
///
/// # Returns
/// A parser matching inline whitespace.
///   
fn whitespace() -> Repeated<impl Parser<char, (), Error = Error> + Copy + Clone> {
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
fn uint(radix: u32) -> impl Parser<char, String, Error = Error> + Copy + Clone {
    filter(move |c: &char| c.is_digit(radix))
        .map(Some)
        .chain::<char, Vec<_>, _>(filter(move |c: &char| c.is_digit(radix)).repeated())
        .collect()
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
) -> impl Parser<char, Box<ParsedExpr>, Error = Error> + 'a
where
    E: Parser<char, ParsedExpr, Error = Error> + 'a,
{
    return text::keyword(cmd)
        .then(whitespace())
        .ignore_then(param_parser)
        .map(Box::from);
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
) -> impl Parser<char, (Box<ParsedExpr>, Box<ParsedExpr>), Error = Error> + 'a
where
    E: Parser<char, ParsedExpr, Error = Error> + 'a,
{
    return text::keyword(cmd)
        .then(whitespace())
        .ignore_then(parser1)
        .then_ignore(just(',').padded_by(whitespace()))
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
) -> impl Parser<
    char,
    (
        Box<ParsedExpr>,
        Box<ParsedExpr>,
        Box<ParsedExpr>,
        Box<ParsedExpr>,
        Box<ParsedExpr>,
    ),
    Error = Error,
> + 'a
where
    E: Parser<char, ParsedExpr, Error = Error> + Clone + 'a,
{
    return text::keyword(cmd)
        .then(whitespace())
        .ignore_then(parser.clone())
        .then_ignore(just(',').padded_by(whitespace()))
        .then(parser.clone())
        .then_ignore(just(',').padded_by(whitespace()))
        .then(parser.clone())
        .then_ignore(just(',').padded_by(whitespace()))
        .then(parser.clone())
        .then_ignore(just(',').padded_by(whitespace()))
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
) -> impl Parser<char, Vec<ParsedExpr>, Error = Error> + 'a
where
    E: Parser<char, Vec<ParsedExpr>, Error = Error> + 'a,
{
    text::keyword(cmd)
        .then(whitespace())
        .ignore_then(param_parser)
}

////////////////////////////////////////////////////////////////

fn parser() -> impl Parser<char, Vec<ParsedExpr>, Error = Error> {
    // TODO: Allow escaped string delimeters within strings. (Don't think it was allowed on original
    // runtest but would be nice to have).
    let string = filter(|c| *c != '"')
        .repeated()
        .delimited_by(just('"'), just('"'))
        .map(String::from_iter)
        .map(Expr::String);

    let uint_dec = uint(10).map(|s: String| Expr::UInt(s.parse().unwrap()));
    let uint_hex = just("$")
        .ignore_then(uint(16))
        .map(|s: String| Expr::UInt(u32::from_str_radix(&s, 16).unwrap()));

    let uint = choice((uint_dec, uint_hex));

    let expr = choice((string, uint))
        .map_with_span(ParsedExpr::from_kind_and_span)
        .padded_by(whitespace());
    let multi_expr = expr.separated_by(just(',').padded_by(whitespace()));

    let string_arg = expr.validate(|arg, span, emit| {
        if !matches!(arg.expresssion(), Expr::String(_)) {
            let expected = [ExprKind::String];
            let found = ExprKind::from(arg.expresssion());

            emit(Error::argument_type(span, expected, found).with_help(
                "If the argument was intended to be a string it should be delimited by \"\"",
            ))
        }

        arg
    });

    let uint_arg = expr.validate(|arg, span, emit| {
        if !matches!(arg.expresssion(), Expr::UInt(_)) {
            let expected = [ExprKind::UInt];
            let found = ExprKind::from(arg.expresssion());

            let mut error = Error::argument_type(span, expected, found);

            if let Expr::String(string) = arg.expresssion() {
                if string.chars().all(|c| c.is_numeric()) {
                    error = error.with_help("If the argument was intended to be an unsigned integer, try removing the enclosing \"\"");
                } else if string.starts_with('$') && string.chars().skip(1).all(|c| c.is_ascii_hexdigit()) {
                    error = error.with_help("If the argument was intended to be a hex unsigned integer, try removing the enclosing \"\"");
                }
            }
            emit(error)
        }

        arg
    });

    let byte_arg = uint_arg.validate(|arg, span, emit| {
        if let Expr::UInt(value) = arg.expresssion() {
            if *value > 255 {
                emit(Error::argument_value_size(span, *value, (0, 255)))
            }
        }

        arg
    });

    ////////////////

    let script_comment = just(';')
        .ignore_then(take_until(choice((newline(), end())).rewind()))
        .map(|(s, _)| String::from_iter(s))
        .map(Expr::ScriptComment)
        .map_with_span(ParsedExpr::from_kind_and_span)
        .padded_by(whitespace());

    ////////////////

    let hpmode = text::keyword("HPMODE").to(Expr::HPMode);
    let comment = command_with_param("COMMENT", string_arg).map(Expr::Comment);
    let wait = command_with_param("WAIT", uint_arg).map(Expr::Wait);
    let opendialog = command_with_param("OPENDIALOG", string_arg).map(Expr::OpenDialog);
    let waitdialog = command_with_param("WAITDIALOG", string_arg).map(Expr::WaitDialog);
    let flush = text::keyword("FLUSH").to(Expr::Flush);
    let protocol = text::keyword("PROTOCOL").to(Expr::Protocol);
    let print = command_with_params("PRINT", multi_expr).map(Expr::Print);
    let settimeformat = command_with_param("SETTIMEFORMAT", byte_arg).map(Expr::SetTimeFormat);
    let settime = text::keyword("SETTIME").to(Expr::SetTime);
    let setoption = command_with_2_params("SETOPTION", byte_arg, byte_arg)
        .map(|(option, setting)| Expr::SetOption { option, setting });

    let tcuclose = command_with_param("TCUCLOSE", byte_arg).map(Expr::TCUClose);
    let tcuopen = command_with_param("TCUOPEN", byte_arg).map(Expr::TCUOpen);
    let tcutest =
        command_with_5_params("TCUTEST", expr).map(|(channel, min, max, retries, message)| {
            Expr::TCUTest {
                channel,
                min,
                max,
                retries,
                message,
            }
        });

    let printerset = command_with_param("PRINTERSET", byte_arg).map(Expr::PrinterSet);
    let printertest =
        command_with_5_params("PRINTERTEST", expr).map(|(channel, min, max, retries, message)| {
            Expr::PrinterTest {
                channel,
                min,
                max,
                retries,
                message,
            }
        });

    let usbopen = text::keyword("USBOPEN").to(Expr::USBOpen);
    let usbclose = text::keyword("USBCLOSE").to(Expr::USBClose);
    let usbprint = command_with_params("USBPRINT", multi_expr).map(Expr::USBPrint);
    let usbsettimeformat =
        command_with_param("USBSETTIMEFORMAT", byte_arg).map(Expr::USBSetTimeFormat);
    let usbsettime = text::keyword("USBSETTIME").to(Expr::USBSetTime);
    let usbsetoption = command_with_2_params("USBSETOPTION", byte_arg, byte_arg)
        .map(|(option, setting)| Expr::USBSetOption { option, setting });

    let usbprinterset = command_with_param("USBPRINTERSET", byte_arg).map(Expr::USBPrinterSet);
    let usbprintertest = command_with_5_params("USBPRINTERTEST", expr).map(
        |(channel, min, max, retries, message)| Expr::USBPrinterTest {
            channel,
            min,
            max,
            retries,
            message,
        },
    );

    let command = choice((
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
    .map_with_span(ParsedExpr::from_kind_and_span)
    .padded_by(whitespace());

    ////////////////

    choice((command, expr, script_comment))
        .separated_by(text::newline().repeated())
        .padded()
        .then_ignore(end())
        .map_err(|error| {
            if let ErrorReason::Unexpected { span, .. } = error.reason() {
                return Error::unrecognised_command(span.clone());
            }

            error
        })
}

////////////////////////////////////////////////////////////////
/// tests
////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::io::Write;

    use ariadne::Source;

    use super::*;

    ////////////////////////////////////////////////////////////////

    fn print_error_reports(script: &str, errors: &[Error]) {
        for error in errors {
            let report = {
                let report = error.to_report();
                let mut string = Vec::new();

                report.write(Source::from(script), string.by_ref()).unwrap();

                String::from_utf8(string)
            };

            if let Ok(report) = report {
                panic!("Parser returned error: {}", report)
            } else {
                panic!("Parser returned unknown error: {:#?}", report)
            }
        }
    }

    ////////////////////////////////////////////////////////////////

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
SETOPTION 4, 6
TCUCLOSE 4
TCUOPEN $F
TCUTEST 5, 12000, 56000, 0, "error"
PRINTERSET 1
PRINTERTEST 4,133, 987,5,"error message"
USBOPEN
USBCLOSE
USBPRINT "Look at me I can print"
USBSETTIMEFORMAT 5
USBSETTIME
USBSETOPTION 5, 9
USBPRINTERSET 6
USBPRINTERTEST 4, 133, 987, 5, "error message"
        "#;

        let parsed_ast = parser().parse(script);

        let expected_ast = [
            Expr::HPMode,
            Expr::Comment(Box::new(ParsedExpr::from_str_default("Test"))),
            Expr::Wait(Box::new(ParsedExpr::from_uint_default(1234))),
            Expr::OpenDialog(Box::new(ParsedExpr::from_str_default("Hello"))),
            Expr::WaitDialog(Box::new(ParsedExpr::from_str_default("PLEASE WAIT"))),
            Expr::Flush,
            Expr::Protocol,
            Expr::Print(vec![ParsedExpr::from_str_default("print me")]),
            Expr::SetTimeFormat(Box::new(ParsedExpr::from_uint_default(0xA6))),
            Expr::SetTime,
            Expr::SetOption {
                option: Box::new(ParsedExpr::from_uint_default(4)),
                setting: Box::new(ParsedExpr::from_uint_default(6)),
            },
            Expr::TCUClose(Box::new(ParsedExpr::from_uint_default(4))),
            Expr::TCUOpen(Box::new(ParsedExpr::from_uint_default(0xF))),
            Expr::TCUTest {
                channel: Box::new(ParsedExpr::from_uint_default(5)),
                min: Box::new(ParsedExpr::from_uint_default(12000)),
                max: Box::new(ParsedExpr::from_uint_default(56000)),
                retries: Box::new(ParsedExpr::from_uint_default(0)),
                message: Box::new(ParsedExpr::from_str_default("error")),
            },
            Expr::PrinterSet(Box::new(ParsedExpr::from_uint_default(1))),
            Expr::PrinterTest {
                channel: Box::new(ParsedExpr::from_uint_default(4)),
                min: Box::new(ParsedExpr::from_uint_default(133)),
                max: Box::new(ParsedExpr::from_uint_default(987)),
                retries: Box::new(ParsedExpr::from_uint_default(5)),
                message: Box::new(ParsedExpr::from_str_default("error message")),
            },
            Expr::USBOpen,
            Expr::USBClose,
            Expr::USBPrint(vec![ParsedExpr::from_str_default("Look at me I can print")]),
            Expr::USBSetTimeFormat(Box::new(ParsedExpr::from_uint_default(5))),
            Expr::USBSetTime,
            Expr::USBSetOption {
                option: Box::new(ParsedExpr::from_uint_default(5)),
                setting: Box::new(ParsedExpr::from_uint_default(9)),
            },
            Expr::USBPrinterSet(Box::new(ParsedExpr::from_uint_default(6))),
            Expr::USBPrinterTest {
                channel: Box::new(ParsedExpr::from_uint_default(4)),
                min: Box::new(ParsedExpr::from_uint_default(133)),
                max: Box::new(ParsedExpr::from_uint_default(987)),
                retries: Box::new(ParsedExpr::from_uint_default(5)),
                message: Box::new(ParsedExpr::from_str_default("error message")),
            },
        ];
        let expected_ast = expected_ast.map(ParsedExpr::from_kind_default);

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
                        let mut string = Vec::new();
                        error
                            .to_report()
                            .write(Source::from(script), string.by_ref())
                            .unwrap();

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

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_single_command() {
        let script = r#"COMMENT "This is a comment 1234""#;

        match parse_from_str(script) {
            Ok(exprs) => {
                assert_eq!(exprs.len(), 1);
                assert_eq!(
                    exprs[0],
                    ParsedExpr::from_kind_default(Expr::Comment(Box::new(
                        ParsedExpr::from_str_default("This is a comment 1234")
                    )))
                )
            }
            Err(errors) => panic!("{:?}", errors),
        }
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_hex_arg_leading_0s() {
        let script = r#"TCUOPEN $00C"#;

        match parse_from_str(script) {
            Ok(exprs) => {
                assert_eq!(exprs.len(), 1);
                assert_eq!(
                    exprs[0],
                    ParsedExpr::from_kind_default(Expr::TCUOpen(Box::new(
                        ParsedExpr::from_uint_default(0x0C)
                    )))
                )
            }
            Err(errors) => print_error_reports(script, &errors),
        }
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_hex_arg_0() {
        let script = r#"TCUOPEN $00"#;

        match parse_from_str(script) {
            Ok(exprs) => {
                assert_eq!(exprs.len(), 1);
                assert_eq!(
                    exprs[0],
                    ParsedExpr::from_kind_default(Expr::TCUOpen(Box::new(
                        ParsedExpr::from_uint_default(0)
                    )))
                )
            }
            Err(errors) => print_error_reports(script, &errors),
        }
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_dec_arg_0() {
        let script = r#"TCUOPEN 0"#;

        match parse_from_str(script) {
            Ok(exprs) => {
                assert_eq!(exprs.len(), 1);
                assert_eq!(
                    exprs[0],
                    ParsedExpr::from_kind_default(Expr::TCUOpen(Box::new(
                        ParsedExpr::from_uint_default(0)
                    )))
                )
            }
            Err(errors) => print_error_reports(script, &errors),
        }
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_invalid_string_type_arg() {
        let script = r#"COMMENT 1234"#;

        let parsed_ast = parser().parse(script);

        match parsed_ast {
            Ok(_) => {}
            Err(errors) => {
                assert_eq!(errors.len(), 1);

                #[allow(unreachable_code)]
                if let Some(error) = errors.first() {
                    assert!(matches!(error.reason(), ErrorReason::ArgType { .. }));
                    return;

                    let report = {
                        let report = error.to_report();
                        let mut string = Vec::new();

                        report.write(Source::from(script), string.by_ref()).unwrap();

                        String::from_utf8(string)
                    };

                    if let Ok(report) = report {
                        panic!("Parser returned error: {}", report)
                    } else {
                        panic!("Parser returned unknown error: {:#?}", report)
                    }
                }
            }
        }
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_invalid_uint_type_arg() {
        let script = r#"WAIT "$F54A""#;

        let parsed_ast = parser().parse(script);

        match parsed_ast {
            Ok(_) => {}
            Err(errors) => {
                assert_eq!(errors.len(), 1);

                #[allow(unreachable_code)]
                if let Some(error) = errors.first() {
                    assert!(matches!(error.reason(), ErrorReason::ArgType { .. }));
                    return;

                    let report = {
                        let report = error.to_report();
                        let mut string = Vec::new();

                        report.write(Source::from(script), string.by_ref()).unwrap();

                        String::from_utf8(string)
                    };

                    if let Ok(report) = report {
                        panic!("Parser returned error: {}", report)
                    } else {
                        panic!("Parser returned unknown error: {:#?}", report)
                    }
                }
            }
        }
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_invalid_uint_value_arg() {
        let script = r#"TCUCLOSE 256"#;

        let parsed_ast = parser().parse(script);

        match parsed_ast {
            Ok(_) => {}
            Err(errors) => {
                assert_eq!(errors.len(), 1);

                #[allow(unreachable_code)]
                if let Some(error) = errors.first() {
                    assert!(matches!(error.reason(), ErrorReason::ArgValue { .. }));
                    return;

                    let report = {
                        let report = error.to_report();
                        let mut string = Vec::new();

                        report.write(Source::from(script), string.by_ref()).unwrap();

                        String::from_utf8(string)
                    };

                    if let Ok(report) = report {
                        panic!("Parser returned error: {}", report)
                    } else {
                        panic!("Parser returned unknown error: {:#?}", report)
                    }
                }
            }
        }
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_comment_own_line() {
        let script = r#";Test comment"#;
        let parsed_ast = parser().parse(script);

        match parsed_ast {
            Ok(ast) => {
                assert_eq!(ast.len(), 1);
                let expr = ast.first().unwrap();
                assert_eq!(
                    *expr.expresssion(),
                    Expr::ScriptComment("Test comment".to_owned())
                )
            }

            Err(errors) => print_error_reports(script, &errors),
        }
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_comment_around_command() {
        let script = r#"
;Comment
PRINT "test" ; Comment
;Comment
        "#;
        let parsed_ast = parser().parse(script);

        match parsed_ast {
            Ok(ast) => {
                assert_eq!(ast.len(), 4);
                let expr = &ast[0];
                assert_eq!(
                    *expr.expresssion(),
                    Expr::ScriptComment("Comment".to_owned())
                );

                let expr = &ast[1];
                assert_eq!(
                    *expr.expresssion(),
                    Expr::Print(vec![ParsedExpr::from_str_default("test")])
                );

                let expr = &ast[2];
                assert_eq!(
                    *expr.expresssion(),
                    Expr::ScriptComment(" Comment".to_owned())
                );

                let expr = &ast[3];
                assert_eq!(
                    *expr.expresssion(),
                    Expr::ScriptComment("Comment".to_owned())
                );
            }

            Err(errors) => print_error_reports(script, &errors),
        }
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_comment_repeated() {
        let script = r#"
;;;;;;Comment
; Comment ;;;; Comment ;;;
;;;;Comment;;;
        "#;
        let parsed_ast = parser().parse(script);

        match parsed_ast {
            Ok(ast) => {
                assert_eq!(ast.len(), 3);
                let expr = &ast[0];
                assert_eq!(
                    *expr.expresssion(),
                    Expr::ScriptComment(";;;;;Comment".to_owned())
                );

                let expr = &ast[1];
                assert_eq!(
                    *expr.expresssion(),
                    Expr::ScriptComment(" Comment ;;;; Comment ;;;".to_owned())
                );

                let expr = &ast[2];
                assert_eq!(
                    *expr.expresssion(),
                    Expr::ScriptComment(";;;Comment;;;".to_owned())
                );
            }

            Err(errors) => print_error_reports(script, &errors),
        }
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_commented_out_command() {
        let script = r#"; PRINT "test""#;
        let parsed_ast = parser().parse(script);

        match parsed_ast {
            Ok(ast) => {
                assert_eq!(ast.len(), 1);
                let expr = ast.first().unwrap();
                assert_eq!(
                    *expr.expresssion(),
                    Expr::ScriptComment(" PRINT \"test\"".to_owned())
                )
            }

            Err(errors) => print_error_reports(script, &errors),
        }
    }
}

////////////////////////////////////////////////////////////////
