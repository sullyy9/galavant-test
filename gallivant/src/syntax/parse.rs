use chumsky::prelude::*;

use super::{
    error::{Error, ErrorReason},
    expression::{parse, ExprKind, ParsedExpr},
};

////////////////////////////////////////////////////////////////

pub fn parse_from_str(script: &str) -> Result<Vec<ParsedExpr>, Vec<Error>> {
    parser().parse(script)
}

////////////////////////////////////////////////////////////////

fn parser() -> impl Parser<char, Vec<ParsedExpr>, Error = Error> {
    let command = choice((
        ExprKind::HPMode.parser(),
        ExprKind::Comment.parser(),
        ExprKind::Wait.parser(),
        ExprKind::OpenDialog.parser(),
        ExprKind::WaitDialog.parser(),
        ExprKind::Flush.parser(),
        ExprKind::Protocol.parser(),
        ExprKind::Print.parser(),
        ExprKind::SetTimeFormat.parser(),
        ExprKind::SetTime.parser(),
        ExprKind::SetOption.parser(),
        ExprKind::TCUClose.parser(),
        ExprKind::TCUOpen.parser(),
        ExprKind::TCUTest.parser(),
        ExprKind::PrinterSet.parser(),
        ExprKind::PrinterTest.parser(),
        // ExprKind::IssueTest.parser(),
        // ExprKind::TestResult.parser(),
        ExprKind::USBOpen.parser(),
        ExprKind::USBClose.parser(),
        ExprKind::USBPrint.parser(),
        ExprKind::USBSetTimeFormat.parser(),
        ExprKind::USBSetTime.parser(),
        ExprKind::USBSetOption.parser(),
        ExprKind::USBPrinterSet.parser(),
        ExprKind::USBPrinterTest.parser(),
    ))
    .padded_by(parse::whitespace());

    ////////////////

    choice((
        command,
        ExprKind::UInt.parser(),
        ExprKind::String.parser(),
        ExprKind::ScriptComment.parser(),
    ))
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
    use crate::syntax::Expr;

    use super::*;

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

        assert_eq!(
            parse_from_str(script).unwrap(),
            [
                Expr::HPMode.into(),
                Expr::Comment(Expr::String("Test".to_owned()).into()).into(),
                Expr::Wait(Expr::UInt(1234).into()).into(),
                Expr::OpenDialog(Expr::String("Hello".to_owned()).into()).into(),
                Expr::WaitDialog(Expr::String("PLEASE WAIT".to_owned()).into()).into(),
                Expr::Flush.into(),
                Expr::Protocol.into(),
                Expr::Print(vec![Expr::String("print me".to_owned()).into()]).into(),
                Expr::SetTimeFormat(Expr::UInt(0xA6).into()).into(),
                Expr::SetTime.into(),
                Expr::SetOption {
                    option: Expr::UInt(4).into(),
                    setting: Expr::UInt(6).into(),
                }
                .into(),
                Expr::TCUClose(Expr::UInt(4).into()).into(),
                Expr::TCUOpen(Expr::UInt(0xF).into()).into(),
                Expr::TCUTest {
                    channel: Expr::UInt(5).into(),
                    min: Expr::UInt(12000).into(),
                    max: Expr::UInt(56000).into(),
                    retries: Expr::UInt(0).into(),
                    message: Expr::String("error".to_owned()).into(),
                }
                .into(),
                Expr::PrinterSet(Expr::UInt(1).into()).into(),
                Expr::PrinterTest {
                    channel: Expr::UInt(4).into(),
                    min: Expr::UInt(133).into(),
                    max: Expr::UInt(987).into(),
                    retries: Expr::UInt(5).into(),
                    message: Expr::String("error message".to_owned()).into(),
                }
                .into(),
                Expr::USBOpen.into(),
                Expr::USBClose.into(),
                Expr::USBPrint(vec![
                    Expr::String("Look at me I can print".to_owned()).into()
                ])
                .into(),
                Expr::USBSetTimeFormat(Expr::UInt(5).into()).into(),
                Expr::USBSetTime.into(),
                Expr::USBSetOption {
                    option: Expr::UInt(5).into(),
                    setting: Expr::UInt(9).into(),
                }
                .into(),
                Expr::USBPrinterSet(Expr::UInt(6).into()).into(),
                Expr::USBPrinterTest {
                    channel: Expr::UInt(4).into(),
                    min: Expr::UInt(133).into(),
                    max: Expr::UInt(987).into(),
                    retries: Expr::UInt(5).into(),
                    message: Expr::String("error message".to_owned()).into(),
                }
                .into(),
            ]
        );
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_single_command() {
        let script = r#"COMMENT "Comment 1234""#;
        assert_eq!(
            parse_from_str(script).unwrap(),
            [Expr::Comment(Expr::String("Comment 1234".to_owned()).into()).into()]
        );
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_hex_arg_leading_0s() {
        let script = r#"TCUOPEN $00C"#;
        assert_eq!(
            parse_from_str(script).unwrap(),
            [Expr::TCUOpen(Expr::UInt(0x0C).into()).into()]
        );
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_hex_arg_0() {
        let script = r#"TCUOPEN $00"#;
        assert_eq!(
            parse_from_str(script).unwrap(),
            [Expr::TCUOpen(Expr::UInt(0).into()).into()]
        );
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_dec_arg_0() {
        let script = r#"TCUOPEN 0"#;
        assert_eq!(
            parse_from_str(script).unwrap(),
            [Expr::TCUOpen(Expr::UInt(0).into()).into()]
        );
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_invalid_string_type_arg() {
        let script = r#"COMMENT 1234"#;

        let errors = parser().parse(script).unwrap_err();

        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors.first().unwrap().reason(),
            ErrorReason::ArgType { .. }
        ));
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_invalid_uint_type_arg() {
        let script = r#"WAIT "$F54A""#;

        let errors = parser().parse(script).unwrap_err();

        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors.first().unwrap().reason(),
            ErrorReason::ArgType { .. }
        ));
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_invalid_uint_value_arg() {
        let script = r#"TCUCLOSE 256"#;

        let errors = parser().parse(script).unwrap_err();

        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors.first().unwrap().reason(),
            ErrorReason::ArgValue { .. }
        ));
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_comment_own_line() {
        let script = r#";Test comment"#;
        assert_eq!(
            parse_from_str(script).unwrap(),
            [Expr::ScriptComment("Test comment".to_owned()).into()]
        );
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_comment_around_command() {
        let script = r#"
;Comment
PRINT "test" ; Comment
;Comment
        "#;

        assert_eq!(
            parse_from_str(script).unwrap(),
            [
                Expr::ScriptComment("Comment".to_owned()).into(),
                Expr::Print(vec![Expr::String("test".to_owned()).into()]).into(),
                Expr::ScriptComment(" Comment".to_owned()).into(),
                Expr::ScriptComment("Comment".to_owned()).into(),
            ]
        );
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_comment_repeated() {
        let script = r#"
;;;;;;Comment
; Comment ;;;; Comment ;;;
;;;;Comment;;;
        "#;

        assert_eq!(
            parse_from_str(script).unwrap(),
            [
                Expr::ScriptComment(";;;;;Comment".to_owned()).into(),
                Expr::ScriptComment(" Comment ;;;; Comment ;;;".to_owned()).into(),
                Expr::ScriptComment(";;;Comment;;;".to_owned()).into(),
            ]
        );
    }

    ////////////////////////////////////////////////////////////////

    #[test]
    fn test_commented_out_command() {
        let script = r#"; PRINT "test""#;

        assert_eq!(
            parse_from_str(script).unwrap(),
            [Expr::ScriptComment(" PRINT \"test\"".to_owned()).into(),]
        );
    }
}

////////////////////////////////////////////////////////////////
