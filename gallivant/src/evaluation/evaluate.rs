use std::time::Duration;

use chrono::{Datelike, Local, Timelike};

use crate::{
    error::Error,
    expression::{Expr, ExprKind},
};

use super::{
    frontend::{Dialog, FrontendRequest, Transaction},
    measurement::MeasurementTest,
    state::ScriptState,
};

////////////////////////////////////////////////////////////////

/// Format a byte into a hex representation using ascii characters. Return those characters as
/// bytes.
///
fn tcu_format_byte(byte: u8) -> Vec<u8> {
    format!("{byte:02X}").into_bytes()
}

////////////////////////////////////////////////////////////////

pub fn evaluate(expr: &Expr, state: &mut ScriptState) -> Result<FrontendRequest, Error> {
    match expr.kind() {
        ExprKind::String(_) => panic!("Orphaned String"),
        ExprKind::UInt(_) => panic!("Orphaned UInt"),

        ExprKind::ScriptComment(_) => Ok(FrontendRequest::None),

        ExprKind::HPMode => {
            state.hpmode = !state.hpmode;
            Ok(FrontendRequest::None)
        }
        ExprKind::Comment(arg) => {
            if let ExprKind::String(str) = arg.kind() {
                return Ok(FrontendRequest::GuiPrint(str.to_owned()));
            }

            panic!("Invalid COMMENT arg {:?}", arg);
        }

        ExprKind::Wait(arg) => {
            if let ExprKind::UInt(milliseconds) = arg.kind() {
                return Ok(FrontendRequest::Wait(Duration::from_millis(
                    (*milliseconds).into(),
                )));
            }

            panic!("Invalid WAIT arg {:?}", arg);
        }

        ExprKind::OpenDialog(arg) => {
            if let ExprKind::String(message) = arg.kind() {
                let kind = Dialog::Notification;
                let message = message.to_owned();
                return Ok(FrontendRequest::GuiDialogue { kind, message });
            }

            panic!("Invalid OPENDIALOG arg {:?}", arg);
        }

        ExprKind::WaitDialog(arg) => {
            if let ExprKind::String(message) = arg.kind() {
                let kind = Dialog::ManualInput;
                let message = message.to_owned();
                return Ok(FrontendRequest::GuiDialogue { kind, message });
            }

            panic!("Invalid WAITDIALOG arg {:?}", arg);
        }

        ExprKind::Flush => Ok(FrontendRequest::TCUFlush),
        ExprKind::Protocol => Ok(FrontendRequest::None),

        ExprKind::Print(args) => {
            let mut arg_bytes = Vec::new();
            for arg in args {
                if let ExprKind::String(str) = arg.kind() {
                    arg_bytes.extend_from_slice(str.as_bytes());
                } else if let ExprKind::UInt(uint) = arg.kind() {
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

            Ok(FrontendRequest::TCUTransact(Transaction::tcu(
                expr.to_owned(),
                bytes,
            )))
        }

        ExprKind::SetTimeFormat(arg) => {
            if let ExprKind::UInt(uint) = arg.kind() {
                let mut bytes = if state.hpmode {
                    Vec::from("P051B007466".as_bytes())
                } else {
                    Vec::from("P051B7466".as_bytes())
                };

                bytes.extend_from_slice(&tcu_format_byte(*uint as u8));
                bytes.push(b'\r');

                return Ok(FrontendRequest::TCUTransact(Transaction::tcu(
                    expr.to_owned(),
                    bytes,
                )));
            }

            panic!()
        }

        ExprKind::SetTime => {
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

            Ok(FrontendRequest::TCUTransact(Transaction::tcu(
                expr.to_owned(),
                bytes,
            )))
        }

        ExprKind::SetOption { option, setting } => {
            if let (ExprKind::UInt(option), ExprKind::UInt(setting)) =
                (option.kind(), setting.kind())
            {
                debug_assert!(*option <= 255);
                debug_assert!(*setting <= 255);

                let bytes = if state.hpmode {
                    format!("P061B00004F{:02X}{:02X}\r", option, setting).into_bytes()
                } else {
                    format!("P061B004F{:02X}{:02X}\r", option, setting).into_bytes()
                };

                return Ok(FrontendRequest::TCUTransact(Transaction::tcu(
                    expr.to_owned(),
                    bytes,
                )));
            }

            panic!("Invalid SETOPTION args {option:?}, {setting:?}")
        }

        ExprKind::TCUClose(arg) => {
            if let ExprKind::UInt(relay) = arg.kind() {
                debug_assert!(*relay <= 255);
                return Ok(FrontendRequest::TCUTransact(Transaction::tcu(
                    expr.to_owned(),
                    format!("C{:02X}\r", relay).into_bytes(),
                )));
            }

            panic!("Invalid TCUCLOSE arg {arg:?}")
        }

        ExprKind::TCUOpen(arg) => {
            if let ExprKind::UInt(relay) = arg.kind() {
                debug_assert!(*relay <= 255);
                return Ok(FrontendRequest::TCUTransact(Transaction::tcu(
                    expr.to_owned(),
                    format!("O{:02X}\r", relay).into_bytes(),
                )));
            }

            panic!("Invalid TCUOPEN arg {arg:?}")
        }

        ExprKind::TCUTest {
            channel,
            min,
            max,
            retries,
            message,
        } => {
            let args = (
                channel.kind(),
                min.kind(),
                max.kind(),
                retries.kind(),
                message.kind(),
            );
            if let (
                ExprKind::UInt(channel),
                ExprKind::UInt(min),
                ExprKind::UInt(max),
                ExprKind::UInt(retries),
                ExprKind::String(message),
            ) = args
            {
                debug_assert!(*channel <= 255);

                return Ok(FrontendRequest::TCUTransact(Transaction::tcu_with_test(
                    expr.clone(),
                    format!("M{channel:02X}\r").into_bytes(),
                    MeasurementTest {
                        expected: *min..=*max,
                        retries: *retries,
                        failure_message: message.to_owned(),
                    },
                )));
            }

            panic!("Invalid TCUTEST args {channel:?}, {min:?}, {max:?}, {retries:?}, {message:?}")
        }

        ExprKind::PrinterSet(arg) => {
            if let ExprKind::UInt(channel) = arg.kind() {
                debug_assert!(*channel <= 255);

                let bytes = if state.hpmode {
                    format!("P051B000053{:02X}\r", channel).into_bytes()
                } else {
                    format!("P051B0053{:02X}\r", channel).into_bytes()
                };

                return Ok(FrontendRequest::TCUTransact(Transaction::tcu(
                    expr.to_owned(),
                    bytes,
                )));
            }

            panic!("Invalid PRINTERSET arg {arg:?}")
        }

        ExprKind::PrinterTest {
            channel,
            min,
            max,
            retries,
            message,
        } => {
            let args = (
                channel.kind(),
                min.kind(),
                max.kind(),
                retries.kind(),
                message.kind(),
            );

            if let (
                ExprKind::UInt(channel),
                ExprKind::UInt(min),
                ExprKind::UInt(max),
                ExprKind::UInt(retries),
                ExprKind::String(message),
            ) = args
            {
                debug_assert!(*channel <= 255);

                let bytes = if state.hpmode {
                    format!("W051B00004D{channel:02X}\r").into_bytes()
                } else {
                    format!("W051B004D{channel:02X}\r").into_bytes()
                };

                return Ok(FrontendRequest::TCUTransact(Transaction::tcu_with_test(
                    expr.clone(),
                    bytes,
                    MeasurementTest {
                        expected: *min..=*max,
                        retries: *retries,
                        failure_message: message.to_owned(),
                    },
                )));
            }

            panic!(
                "Invalid PRINTERTEST args {channel:?}, {min:?}, {max:?}, {retries:?}, {message:?}"
            )
        }

        ExprKind::IssueTest(_) => Ok(FrontendRequest::None),
        ExprKind::TestResult { .. } => Ok(FrontendRequest::None),

        ExprKind::USBOpen => Ok(FrontendRequest::PrinterOpen),
        ExprKind::USBClose => Ok(FrontendRequest::PrinterClose),

        ExprKind::USBPrint(args) => {
            let mut bytes = Vec::new();
            for arg in args {
                if let ExprKind::String(str) = arg.kind() {
                    bytes.extend_from_slice(str.as_bytes());
                } else if let ExprKind::UInt(uint) = arg.kind() {
                    debug_assert!(*uint <= 255);
                    bytes.push(*uint as u8);
                } else {
                    panic!("Invalid USBPRINT arg {arg:?}")
                }
            }

            Ok(FrontendRequest::PrinterTransmit(bytes))
        }

        ExprKind::USBSetTimeFormat(arg) => {
            if let ExprKind::UInt(uint) = arg.kind() {
                let bytes = if state.hpmode {
                    vec![0x1B, 0x00, b't', b'f', *uint as u8]
                } else {
                    vec![0x1B, b't', b'f', *uint as u8]
                };

                return Ok(FrontendRequest::PrinterTransmit(bytes));
            }

            panic!()
        }

        ExprKind::USBSetTime => {
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

            Ok(FrontendRequest::PrinterTransmit(bytes))
        }

        ExprKind::USBSetOption { option, setting } => {
            if let (ExprKind::UInt(option), ExprKind::UInt(setting)) =
                (option.kind(), setting.kind())
            {
                debug_assert!(*option <= 255);
                debug_assert!(*setting <= 255);

                let bytes = if state.hpmode {
                    vec![0x1B, 0x00, 0x00, b'O', *option as u8, *setting as u8]
                } else {
                    vec![0x1B, 0x00, b'O', *option as u8, *setting as u8]
                };

                return Ok(FrontendRequest::PrinterTransmit(bytes));
            }

            panic!("Invalid USBSETOPTION args {option:?}, {setting:?}")
        }

        ExprKind::USBPrinterSet(arg) => {
            if let ExprKind::UInt(channel) = arg.kind() {
                debug_assert!(*channel <= 255);

                let bytes = if state.hpmode {
                    vec![0x1B, 0x00, 0x00, b'S', *channel as u8]
                } else {
                    vec![0x1B, 0x00, b'S', *channel as u8]
                };

                return Ok(FrontendRequest::PrinterTransmit(bytes));
            }

            panic!("Invalid USBPRINTERSET arg {arg:?}")
        }

        ExprKind::USBPrinterTest {
            channel,
            min,
            max,
            retries,
            message,
        } => {
            let args = (
                channel.kind(),
                min.kind(),
                max.kind(),
                retries.kind(),
                message.kind(),
            );

            if let (
                ExprKind::UInt(channel),
                ExprKind::UInt(min),
                ExprKind::UInt(max),
                ExprKind::UInt(retries),
                ExprKind::String(message),
            ) = args
            {
                debug_assert!(*channel <= 255);

                let bytes = if state.hpmode {
                    vec![0x1B, 0x00, 0x00, b'M', *channel as u8]
                } else {
                    vec![0x1B, 0x00, b'M', *channel as u8]
                };

                return Ok(FrontendRequest::PrinterTransact(
                    Transaction::printer_with_test(
                        expr.clone(),
                        bytes,
                        MeasurementTest {
                            expected: *min..=*max,
                            retries: *retries,
                            failure_message: message.to_owned(),
                        },
                    ),
                ));
            }

            panic!(
                "Invalid USBPRINTERTEST args {channel:?}, {min:?}, {max:?}, {retries:?}, {message:?}"
            )
        }
    }
}

////////////////////////////////////////////////////////////////
