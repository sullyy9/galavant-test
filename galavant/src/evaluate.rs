use std::time::Duration;

use crate::{
    error::Error,
    expression::{Expr, ExprKind},
};

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

// Will need to handle requests that will receive a response. e.g. For the TCUTEST command.
// Maybe pass a callback fn to call with the requests response?
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum FrontendRequest {
    Nothing,
    Wait(Duration),

    GuiPrint(String),
    GuiDialogue { kind: Dialog, message: String },

    TCUTransmit(Vec<u8>),
    TCUFlush,

    PrinterTransmit(Vec<u8>),
}

////////////////////////////////////////////////////////////////

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Dialog {
    Notification { await_close: bool },
}

////////////////////////////////////////////////////////////////

pub fn evaluate(expr: Expr) -> Result<FrontendRequest, Error> {
    type Request = FrontendRequest;

    match expr.kind() {
        ExprKind::String(_) => panic!("Orphaned String"),
        ExprKind::UInt(_) => panic!("Orphaned UInt"),

        ExprKind::HPMode => Ok(Request::Nothing),

        ExprKind::Comment(arg) => {
            if let ExprKind::String(str) = arg.kind() {
                return Ok(Request::GuiPrint(str.to_owned()));
            }

            panic!("Invalid COMMENT arg {:?}", arg);
        }

        ExprKind::Wait(arg) => {
            if let ExprKind::UInt(milliseconds) = arg.kind() {
                return Ok(Request::Wait(Duration::from_millis((*milliseconds).into())));
            }

            panic!("Invalid WAIT arg {:?}", arg);
        }

        ExprKind::OpenDialog(arg) => {
            if let ExprKind::String(message) = arg.kind() {
                let kind = Dialog::Notification { await_close: false };
                let message = message.to_owned();
                return Ok(Request::GuiDialogue { kind, message });
            }

            panic!("Invalid OPENDIALOG arg {:?}", arg);
        }

        ExprKind::WaitDialog(arg) => {
            if let ExprKind::String(message) = arg.kind() {
                let kind = Dialog::Notification { await_close: true };
                let message = message.to_owned();
                return Ok(Request::GuiDialogue { kind, message });
            }

            panic!("Invalid WAITDIALOG arg {:?}", arg);
        }

        ExprKind::Flush => Ok(Request::TCUFlush),
        ExprKind::Protocol => Ok(Request::Nothing),

        ExprKind::Print(args) => {
            let mut arg_bytes = Vec::new();
            for arg in args {
                if let ExprKind::String(str) = arg.kind() {
                    arg_bytes.extend_from_slice(str.as_bytes());
                } else if let ExprKind::UInt(uint) = arg.kind() {
                    debug_assert!(*uint <= 255);
                    arg_bytes.push(*uint as u8);
                }
            }

            // Each byte needs to be transformed into an ascii hex representation.
            let arg_bytes: Vec<u8> = arg_bytes
                .iter()
                .flat_map(|b| format!("{b:02X}").into_bytes())
                .collect();

            let mut bytes = vec![b'P'];
            bytes.extend_from_slice(format!("{:02X}", arg_bytes.len()).as_bytes());
            bytes.extend_from_slice(&arg_bytes);

            Ok(Request::TCUTransmit(bytes))
        }

        ExprKind::SetTimeFormat(_) => todo!(),
        ExprKind::SetTime => todo!(),
        ExprKind::SetOption { option, setting } => todo!(),
        ExprKind::TCUClose(_) => todo!(),
        ExprKind::TCUOpen(_) => todo!(),
        ExprKind::TCUTest {
            channel,
            min,
            max,
            retries,
            message,
        } => todo!(),
        ExprKind::PrinterSet(_) => todo!(),
        ExprKind::PrinterTest {
            channel,
            min,
            max,
            retries,
            message,
        } => todo!(),
        ExprKind::IssueTest(_) => todo!(),
        ExprKind::TestResult { min, max, message } => todo!(),
        ExprKind::USBOpen => todo!(),
        ExprKind::USBClose => todo!(),
        ExprKind::USBPrint(_) => todo!(),
        ExprKind::USBSetTimeFormat(_) => todo!(),
        ExprKind::USBSetTime => todo!(),
        ExprKind::USBSetOption { option, setting } => todo!(),
        ExprKind::USBPrinterSet(_) => todo!(),
        ExprKind::USBPrinterTest {
            channel,
            min,
            max,
            retries,
            message,
        } => todo!(),
    }
}

////////////////////////////////////////////////////////////////
