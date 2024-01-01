use std::time::Duration;

use super::transaction::Transaction;

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

/// Requests for actions a frontend needs to perform during script execution.
///
#[derive(Clone, Debug, PartialEq)]
pub enum FrontendRequest {
    None,
    Wait(Duration),

    GuiPrint(String),
    GuiDialogue { kind: Dialog, message: String },

    TCUTransact(Transaction),
    TCUFlush,

    // Requests for direct communication with the printer i.e. not via the TCU.
    PrinterOpen,
    PrinterClose,
    PrinterTransmit(Vec<u8>),
    PrinterTransact(Transaction),
}

////////////////////////////////////////////////////////////////

/// Types of dialog a frontend may need to create during script execution.
///
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dialog {
    Notification,

    /// Dialog that should display a message and allow the user to either continue or stop the test.
    ManualInput,
}

////////////////////////////////////////////////////////////////
