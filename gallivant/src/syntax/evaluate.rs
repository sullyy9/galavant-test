use std::time::Duration;

use chrono::{Datelike, Local, Timelike};

use crate::execution::{Dialog, FrontendRequest, MeasurementTest, Transaction};

use super::{
    error::Error,
    expression::{Expr, ParsedExpr},
    state::EvalState,
};

////////////////////////////////////////////////////////////////

/// Format a byte into a hex representation using ascii characters. Return those characters as
/// bytes.
///
fn tcu_format_byte(byte: u8) -> Vec<u8> {
    format!("{byte:02X}").into_bytes()
}

////////////////////////////////////////////////////////////////

pub fn evaluate(expr: &ParsedExpr, state: &mut EvalState) -> Result<FrontendRequest, Error> {
    match expr.expression() {
        Expr::String(_) => panic!("Orphaned String"),
        Expr::UInt(_) => panic!("Orphaned UInt"),

        Expr::ScriptComment(_) => Ok(FrontendRequest::None),

        Expr::HPMode => {
            state.hpmode = !state.hpmode;
            Ok(FrontendRequest::None)
        }
        Expr::Comment(arg) => {
            if let Expr::String(str) = arg.expression() {
                return Ok(FrontendRequest::GuiPrint(str.to_owned()));
            }

            panic!("Invalid COMMENT arg {:?}", arg);
        }

        Expr::Wait(arg) => {
            if let Expr::UInt(milliseconds) = arg.expression() {
                return Ok(FrontendRequest::Wait(Duration::from_millis(
                    (*milliseconds).into(),
                )));
            }

            panic!("Invalid WAIT arg {:?}", arg);
        }

        Expr::OpenDialog(arg) => {
            if let Expr::String(message) = arg.expression() {
                let kind = Dialog::Notification;
                let message = message.to_owned();
                return Ok(FrontendRequest::GuiDialogue { kind, message });
            }

            panic!("Invalid OPENDIALOG arg {:?}", arg);
        }

        Expr::WaitDialog(arg) => {
            if let Expr::String(message) = arg.expression() {
                let kind = Dialog::ManualInput;
                let message = message.to_owned();
                return Ok(FrontendRequest::GuiDialogue { kind, message });
            }

            panic!("Invalid WAITDIALOG arg {:?}", arg);
        }

        Expr::Flush => Ok(FrontendRequest::TCUFlush),
        Expr::Protocol => Ok(FrontendRequest::None),

        Expr::Print(args) => {
            let mut arg_bytes = Vec::new();
            for arg in args {
                if let Expr::String(str) = arg.expression() {
                    arg_bytes.extend_from_slice(str.as_bytes());
                } else if let Expr::UInt(uint) = arg.expression() {
                    debug_assert!(*uint <= 255);
                    arg_bytes.push(*uint as u8);
                } else {
                    panic!("Invalid PRINT arg {arg:?}")
                }
            }

            // Each byte needs to be transformed into an ascii hex representation.
            let arg_bytes: Vec<u8> = arg_bytes.into_iter().flat_map(tcu_format_byte).collect();

            if arg_bytes.len() > 255 {
                todo!(
                    "Print command is limited to 255 args due to requirement to transmit the number
                    of args as a byte. Need to return an error here."
                );
            }

            let mut bytes = vec![b'P'];
            bytes.extend_from_slice(&tcu_format_byte(arg_bytes.len() as u8));
            bytes.extend_from_slice(&arg_bytes);
            bytes.push(b'\r');

            Ok(FrontendRequest::TCUTransact(Transaction::with_tcu(
                expr.to_owned(),
                bytes,
                None,
            )))
        }

        Expr::SetTimeFormat(arg) => {
            if let Expr::UInt(uint) = arg.expression() {
                let mut bytes = if state.hpmode {
                    Vec::from("P051B007466".as_bytes())
                } else {
                    Vec::from("P051B7466".as_bytes())
                };

                bytes.extend_from_slice(&tcu_format_byte(*uint as u8));
                bytes.push(b'\r');

                return Ok(FrontendRequest::TCUTransact(Transaction::with_tcu(
                    expr.to_owned(),
                    bytes,
                    None,
                )));
            }

            panic!()
        }

        Expr::SetTime => {
            let datetime = Local::now();
            let datetime = format!(
                "{:02}:{:02}:{:02},{:02}/{:02}/{:02}",
                datetime.hour(),
                datetime.minute(),
                datetime.second(),
                datetime.day(),
                datetime.month(),
                (datetime.year() - 1900) % 100
            );

            let datetime = datetime.into_bytes().into_iter().flat_map(tcu_format_byte);

            let mut bytes = if state.hpmode {
                Vec::from("P151B007473".as_bytes())
            } else {
                Vec::from("P151B7473".as_bytes())
            };

            bytes.extend(datetime);
            bytes.push(b'\r');

            Ok(FrontendRequest::TCUTransact(Transaction::with_tcu(
                expr.to_owned(),
                bytes,
                None,
            )))
        }

        Expr::SetOption { option, setting } => {
            if let (Expr::UInt(option), Expr::UInt(setting)) =
                (option.expression(), setting.expression())
            {
                debug_assert!(*option <= 255);
                debug_assert!(*setting <= 255);

                let bytes = if state.hpmode {
                    format!("P061B00004F{:02X}{:02X}\r", option, setting).into_bytes()
                } else {
                    format!("P061B004F{:02X}{:02X}\r", option, setting).into_bytes()
                };

                return Ok(FrontendRequest::TCUTransact(Transaction::with_tcu(
                    expr.to_owned(),
                    bytes,
                    None,
                )));
            }

            panic!("Invalid SETOPTION args {option:?}, {setting:?}")
        }

        Expr::TCUClose(arg) => {
            if let Expr::UInt(relay) = arg.expression() {
                debug_assert!(*relay <= 255);
                return Ok(FrontendRequest::TCUTransact(Transaction::with_tcu(
                    expr.to_owned(),
                    format!("C{:02X}\r", relay).into_bytes(),
                    None,
                )));
            }

            panic!("Invalid TCUCLOSE arg {arg:?}")
        }

        Expr::TCUOpen(arg) => {
            if let Expr::UInt(relay) = arg.expression() {
                debug_assert!(*relay <= 255);
                return Ok(FrontendRequest::TCUTransact(Transaction::with_tcu(
                    expr.to_owned(),
                    format!("O{:02X}\r", relay).into_bytes(),
                    None,
                )));
            }

            panic!("Invalid TCUOPEN arg {arg:?}")
        }

        Expr::TCUTest {
            channel,
            min,
            max,
            retries,
            message,
        } => {
            let args = (
                channel.expression(),
                min.expression(),
                max.expression(),
                retries.expression(),
                message.expression(),
            );
            if let (
                Expr::UInt(channel),
                Expr::UInt(min),
                Expr::UInt(max),
                Expr::UInt(retries),
                Expr::String(message),
            ) = args
            {
                debug_assert!(*channel <= 255);

                return Ok(FrontendRequest::TCUTransact(Transaction::with_tcu(
                    expr.clone(),
                    format!("M{channel:02X}\r").into_bytes(),
                    Some(MeasurementTest {
                        expected: *min..=*max,
                        retries: *retries,
                        failure_message: message.to_owned(),
                    }),
                )));
            }

            panic!("Invalid TCUTEST args {channel:?}, {min:?}, {max:?}, {retries:?}, {message:?}")
        }

        Expr::PrinterSet(arg) => {
            if let Expr::UInt(channel) = arg.expression() {
                debug_assert!(*channel <= 255);

                let bytes = if state.hpmode {
                    format!("P051B000053{:02X}\r", channel).into_bytes()
                } else {
                    format!("P051B0053{:02X}\r", channel).into_bytes()
                };

                return Ok(FrontendRequest::TCUTransact(Transaction::with_tcu(
                    expr.to_owned(),
                    bytes,
                    None,
                )));
            }

            panic!("Invalid PRINTERSET arg {arg:?}")
        }

        Expr::PrinterTest {
            channel,
            min,
            max,
            retries,
            message,
        } => {
            let args = (
                channel.expression(),
                min.expression(),
                max.expression(),
                retries.expression(),
                message.expression(),
            );

            if let (
                Expr::UInt(channel),
                Expr::UInt(min),
                Expr::UInt(max),
                Expr::UInt(retries),
                Expr::String(message),
            ) = args
            {
                debug_assert!(*channel <= 255);

                let bytes = if state.hpmode {
                    format!("W051B00004D{channel:02X}\r").into_bytes()
                } else {
                    format!("W051B004D{channel:02X}\r").into_bytes()
                };

                return Ok(FrontendRequest::TCUTransact(Transaction::with_tcu(
                    expr.clone(),
                    bytes,
                    Some(MeasurementTest {
                        expected: *min..=*max,
                        retries: *retries,
                        failure_message: message.to_owned(),
                    }),
                )));
            }

            panic!(
                "Invalid PRINTERTEST args {channel:?}, {min:?}, {max:?}, {retries:?}, {message:?}"
            )
        }

        Expr::IssueTest(_) => Ok(FrontendRequest::None),
        Expr::TestResult { .. } => Ok(FrontendRequest::None),

        Expr::USBOpen => Ok(FrontendRequest::PrinterOpen),
        Expr::USBClose => Ok(FrontendRequest::PrinterClose),

        Expr::USBPrint(args) => {
            let mut bytes = Vec::new();
            for arg in args {
                if let Expr::String(str) = arg.expression() {
                    bytes.extend_from_slice(str.as_bytes());
                } else if let Expr::UInt(uint) = arg.expression() {
                    debug_assert!(*uint <= 255);
                    bytes.push(*uint as u8);
                } else {
                    panic!("Invalid USBPRINT arg {arg:?}")
                }
            }

            Ok(FrontendRequest::PrinterTransact(Transaction::with_printer(
                expr.clone(),
                bytes,
                None,
            )))
        }

        Expr::USBSetTimeFormat(arg) => {
            if let Expr::UInt(uint) = arg.expression() {
                let bytes = if state.hpmode {
                    vec![0x1B, 0x00, b't', b'f', *uint as u8]
                } else {
                    vec![0x1B, b't', b'f', *uint as u8]
                };

                return Ok(FrontendRequest::PrinterTransact(Transaction::with_printer(
                    expr.clone(),
                    bytes,
                    None,
                )));
            }

            panic!()
        }

        Expr::USBSetTime => {
            let datetime = Local::now();
            let datetime = format!(
                "{:02}:{:02}:{:02},{:02}/{:02}/{:02}",
                datetime.hour(),
                datetime.minute(),
                datetime.second(),
                datetime.day(),
                datetime.month(),
                (datetime.year() - 1900) % 100
            );

            let mut bytes = if state.hpmode {
                vec![0x1B, 0x00, b't', b's']
            } else {
                vec![0x1B, b't', b's']
            };

            bytes.extend_from_slice(datetime.as_bytes());

            Ok(FrontendRequest::PrinterTransact(Transaction::with_printer(
                expr.clone(),
                bytes,
                None,
            )))
        }

        Expr::USBSetOption { option, setting } => {
            if let (Expr::UInt(option), Expr::UInt(setting)) =
                (option.expression(), setting.expression())
            {
                debug_assert!(*option <= 255);
                debug_assert!(*setting <= 255);

                let bytes = if state.hpmode {
                    vec![0x1B, 0x00, 0x00, b'O', *option as u8, *setting as u8]
                } else {
                    vec![0x1B, 0x00, b'O', *option as u8, *setting as u8]
                };

                return Ok(FrontendRequest::PrinterTransact(Transaction::with_printer(
                    expr.clone(),
                    bytes,
                    None,
                )));
            }

            panic!("Invalid USBSETOPTION args {option:?}, {setting:?}")
        }

        Expr::USBPrinterSet(arg) => {
            if let Expr::UInt(channel) = arg.expression() {
                debug_assert!(*channel <= 255);

                let bytes = if state.hpmode {
                    vec![0x1B, 0x00, 0x00, b'S', *channel as u8]
                } else {
                    vec![0x1B, 0x00, b'S', *channel as u8]
                };

                return Ok(FrontendRequest::PrinterTransact(Transaction::with_printer(
                    expr.clone(),
                    bytes,
                    None,
                )));
            }

            panic!("Invalid USBPRINTERSET arg {arg:?}")
        }

        Expr::USBPrinterTest {
            channel,
            min,
            max,
            retries,
            message,
        } => {
            let args = (
                channel.expression(),
                min.expression(),
                max.expression(),
                retries.expression(),
                message.expression(),
            );

            if let (
                Expr::UInt(channel),
                Expr::UInt(min),
                Expr::UInt(max),
                Expr::UInt(retries),
                Expr::String(message),
            ) = args
            {
                debug_assert!(*channel <= 255);

                let bytes = if state.hpmode {
                    vec![0x1B, 0x00, 0x00, b'M', *channel as u8]
                } else {
                    vec![0x1B, 0x00, b'M', *channel as u8]
                };

                return Ok(FrontendRequest::PrinterTransact(Transaction::with_printer(
                    expr.clone(),
                    bytes,
                    Some(MeasurementTest {
                        expected: *min..=*max,
                        retries: *retries,
                        failure_message: message.to_owned(),
                    }),
                )));
            }

            panic!(
                "Invalid USBPRINTERTEST args {channel:?}, {min:?}, {max:?}, {retries:?}, {message:?}"
            )
        }
    }
}

////////////////////////////////////////////////////////////////
