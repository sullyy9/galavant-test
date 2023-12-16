use chumsky::{prelude::*, text::Character};

type Error = chumsky::error::Simple<char>;

////////////////////////////////////////////////////////////////

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Expr {
    String(String),
    UInt(u32),

    HPMode,
    Comment(Box<Expr>),
    Wait(Box<Expr>),
    OpenDialog(Box<Expr>),
    WaitDialog(Box<Expr>),
    Flush,
    Protocol,
    Print(Vec<Expr>),
    SetTimeFormat(Box<Expr>),
    SetTime,
    SetOption {
        option: Box<Expr>,
        setting: Box<Expr>,
    },
    TCUClose(Box<Expr>),
    TCUOpen(Box<Expr>),
    TCUTest {
        channel: Box<Expr>,
        min: Box<Expr>,
        max: Box<Expr>,
        retries: Box<Expr>,
        message: Box<Expr>,
    },
    PrinterSet(Box<Expr>),
    PrinterTest {
        channel: Box<Expr>,
        min: Box<Expr>,
        max: Box<Expr>,
        retries: Box<Expr>,
        message: Box<Expr>,
    },
    IssueTest(Box<Expr>), // Unused.
    TestResult {
        // Unused.
        min: Box<Expr>,
        max: Box<Expr>,
        message: Box<Expr>,
    },
    USBOpen,
    USBClose,
    USBPrint(Vec<Expr>),
    USBSetTimeFormat(Box<Expr>),
    USBSetTime,
    USBSetOption {
        option: Box<Expr>,
        setting: Box<Expr>,
    },
    USBPrinterSet(Box<Expr>),
    USBPrinterTest {
        channel: Box<Expr>,
        min: Box<Expr>,
        max: Box<Expr>,
        retries: Box<Expr>,
        message: Box<Expr>,
    },
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

pub fn parser() -> impl Parser<char, Vec<Expr>, Error = Error> {
    let whitespace = filter(|c: &char| c.is_inline_whitespace())
        .ignored()
        .repeated();

    let string = just('"')
        .ignore_then(take_until(just('"')))
        .map(|(s, _)| Expr::String(String::from_iter(s)));

    let uint_dec = text::int(10).map(|s: String| Expr::UInt(s.parse().unwrap()));
    let uint_hex = just("$")
        .ignore_then(text::int(16))
        .map(|s: String| Expr::UInt(u32::from_str_radix(&s, 16).unwrap()));

    let uint = uint_dec.or(uint_hex);

    let expr = choice((string, uint)).padded_by(whitespace);
    let multi_expr = expr.separated_by(whitespace);

    ////////////////

    let hpmode = text::keyword("HPMODE").to(Expr::HPMode);
    let comment = command_with_param("COMMENT", expr).map(Expr::Comment);
    let wait = command_with_param("WAIT", expr).map(Expr::Wait);
    let opendialog = command_with_param("OPENDIALOG", expr).map(Expr::OpenDialog);
    let waitdialog = command_with_param("WAITDIALOG", expr).map(Expr::WaitDialog);
    let flush = text::keyword("FLUSH").to(Expr::Flush);
    let protocol = text::keyword("PROTOCOL").to(Expr::Protocol);
    let print = command_with_params("PRINT", multi_expr).map(Expr::Print);
    let settimeformat = command_with_param("SETTIMEFORMAT", expr).map(Expr::SetTimeFormat);
    let settime = text::keyword("SETTIME").to(Expr::SetTime);
    let setoption = command_with_2_params("SETOPTION", expr, expr)
        .map(|(option, setting)| Expr::SetOption { option, setting });

    let tcuclose = command_with_param("TCUCLOSE", expr).map(Expr::TCUClose);
    let tcuopen = command_with_param("TCUOPEN", expr).map(Expr::TCUOpen);
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

    let printerset = command_with_param("PRINTERSET", expr).map(Expr::PrinterSet);
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
    let usbsettimeformat = command_with_param("USBSETTIMEFORMAT", expr).map(Expr::USBSetTimeFormat);
    let usbsettime = text::keyword("USBSETTIME").to(Expr::USBSetTime);
    let usbsetoption = command_with_2_params("USBSETOPTION", expr, expr)
        .map(|(option, setting)| Expr::USBSetOption { option, setting });

    let usbprinterset = command_with_param("USBPRINTERSET", expr).map(Expr::USBPrinterSet);
    let usbprintertest = command_with_5_params("USBPRINTERTEST", expr).map(
        |(channel, min, max, retries, message)| Expr::USBPrinterTest {
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
    .padded_by(whitespace);

    ////////////////

    decl.or(expr)
        .separated_by(text::newline().repeated())
        .padded()
        .then_ignore(end())
}

////////////////////////////////////////////////////////////////

pub fn eval(expr: &Expr) -> Result<(), String> {
    match expr {
        Expr::String(_) => todo!(),
        Expr::UInt(_) => todo!(),

        Expr::HPMode => todo!(),
        Expr::Comment(_) => todo!(),
        Expr::Wait(_) => todo!(),
        Expr::OpenDialog(_) => todo!(),
        Expr::WaitDialog(_) => todo!(),
        Expr::Flush => todo!(),
        Expr::Protocol => todo!(),
        Expr::Print(_) => todo!(),
        Expr::SetTimeFormat(_) => todo!(),
        Expr::SetTime => todo!(),
        Expr::SetOption { option, setting } => todo!(),
        Expr::TCUClose(_) => todo!(),
        Expr::TCUOpen(_) => todo!(),
        Expr::TCUTest {
            channel,
            min,
            max,
            retries,
            message,
        } => todo!(),
        Expr::PrinterSet(_) => todo!(),
        Expr::PrinterTest {
            channel,
            min,
            max,
            retries,
            message,
        } => todo!(),
        Expr::IssueTest(_) => todo!(),
        Expr::TestResult { min, max, message } => todo!(),
        Expr::USBOpen => todo!(),
        Expr::USBClose => todo!(),
        Expr::USBPrint(_) => todo!(),
        Expr::USBSetTimeFormat(_) => todo!(),
        Expr::USBSetTime => todo!(),
        Expr::USBSetOption { option, setting } => todo!(),
        Expr::USBPrinterSet(_) => todo!(),
        Expr::USBPrinterTest {
            channel,
            min,
            max,
            retries,
            message,
        } => todo!(),
    }
}

////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::io::Write;

    use ariadne::{Label, Report, ReportKind, Source};

    use super::*;

    #[test]
    fn test_parse_commands() {
        let script = "
            HPMODE
            COMMENT \"Test\"
            WAIT 1234
            OPENDIALOG \"Hello\"
            WAITDIALOG \"PLEASE WAIT\"
            FLUSH
            PROTOCOL
            PRINT \"print me\"
            SETTIMEFORMAT $A6
            SETTIME
            SETOPTION 4 6
            TCUCLOSE 4
            TCUOPEN $F
            TCUTEST 5 12000 56000 0 \"error\"
            PRINTERSET 1
            PRINTERTEST 4 133 987 5 \"error message\"
            USBOPEN
            USBCLOSE
            USBPRINT \"Look at me I can print\"
            USBSETTIMEFORMAT 5
            USBSETTIME
            USBSETOPTION 5 9
            USBPRINTERSET 6
            USBPRINTERTEST 4 133 987 5 \"error message\"
        ";

        let parsed_ast = parser().parse(script);

        let expected_ast = [
            Expr::HPMode,
            Expr::Comment(Box::new(Expr::String(String::from("Test")))),
            Expr::Wait(Box::new(Expr::UInt(1234))),
            Expr::OpenDialog(Box::new(Expr::String(String::from("Hello")))),
            Expr::WaitDialog(Box::new(Expr::String(String::from("PLEASE WAIT")))),
            Expr::Flush,
            Expr::Protocol,
            Expr::Print(vec![Expr::String(String::from("print me"))]),
            Expr::SetTimeFormat(Box::new(Expr::UInt(0xA6))),
            Expr::SetTime,
            Expr::SetOption {
                option: Box::new(Expr::UInt(4)),
                setting: Box::new(Expr::UInt(6)),
            },
            Expr::TCUClose(Box::new(Expr::UInt(4))),
            Expr::TCUOpen(Box::new(Expr::UInt(0xF))),
            Expr::TCUTest {
                channel: Box::new(Expr::UInt(5)),
                min: Box::new(Expr::UInt(12000)),
                max: Box::new(Expr::UInt(56000)),
                retries: Box::new(Expr::UInt(0)),
                message: Box::new(Expr::String(String::from("error"))),
            },
            Expr::PrinterSet(Box::new(Expr::UInt(1))),
            Expr::PrinterTest {
                channel: Box::new(Expr::UInt(4)),
                min: Box::new(Expr::UInt(133)),
                max: Box::new(Expr::UInt(987)),
                retries: Box::new(Expr::UInt(5)),
                message: Box::new(Expr::String(String::from("error message"))),
            },
            Expr::USBOpen,
            Expr::USBClose,
            Expr::USBPrint(vec![Expr::String(String::from("Look at me I can print"))]),
            Expr::USBSetTimeFormat(Box::new(Expr::UInt(5))),
            Expr::USBSetTime,
            Expr::USBSetOption {
                option: Box::new(Expr::UInt(5)),
                setting: Box::new(Expr::UInt(9)),
            },
            Expr::USBPrinterSet(Box::new(Expr::UInt(6))),
            Expr::USBPrinterTest {
                channel: Box::new(Expr::UInt(4)),
                min: Box::new(Expr::UInt(133)),
                max: Box::new(Expr::UInt(987)),
                retries: Box::new(Expr::UInt(5)),
                message: Box::new(Expr::String(String::from("error message"))),
            },
        ];

        match parsed_ast {
            Ok(ast) => {
                for (i, (actual_expr, expected_expr)) in
                    std::iter::zip(ast, expected_ast).enumerate()
                {
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
}

////////////////////////////////////////////////////////////////
