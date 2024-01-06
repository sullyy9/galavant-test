use std::ops::Range;

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

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    String(String),
    UInt(u32),

    ScriptComment(String),

    HPMode,
    Comment(Box<ParsedExpr>),
    Wait(Box<ParsedExpr>),
    OpenDialog(Box<ParsedExpr>),
    WaitDialog(Box<ParsedExpr>),
    Flush,
    Protocol,
    Print(Vec<ParsedExpr>),
    SetTimeFormat(Box<ParsedExpr>),

    /// This requires getting the current time from the OS and sending it to the printer via the
    /// TCU. Need to consider that the time must be acquired just before the command is sent.
    SetTime,
    SetOption {
        option: Box<ParsedExpr>,
        setting: Box<ParsedExpr>,
    },
    TCUClose(Box<ParsedExpr>),
    TCUOpen(Box<ParsedExpr>),
    TCUTest {
        channel: Box<ParsedExpr>,
        min: Box<ParsedExpr>,
        max: Box<ParsedExpr>,
        retries: Box<ParsedExpr>,
        message: Box<ParsedExpr>,
    },
    PrinterSet(Box<ParsedExpr>),
    PrinterTest {
        channel: Box<ParsedExpr>,
        min: Box<ParsedExpr>,
        max: Box<ParsedExpr>,
        retries: Box<ParsedExpr>,
        message: Box<ParsedExpr>,
    },
    IssueTest(Box<ParsedExpr>), // Unused.
    TestResult {
        // Unused.
        min: Box<ParsedExpr>,
        max: Box<ParsedExpr>,
        message: Box<ParsedExpr>,
    },
    USBOpen,
    USBClose,
    USBPrint(Vec<ParsedExpr>),
    USBSetTimeFormat(Box<ParsedExpr>),
    USBSetTime,
    USBSetOption {
        option: Box<ParsedExpr>,
        setting: Box<ParsedExpr>,
    },
    USBPrinterSet(Box<ParsedExpr>),
    USBPrinterTest {
        channel: Box<ParsedExpr>,
        min: Box<ParsedExpr>,
        max: Box<ParsedExpr>,
        retries: Box<ParsedExpr>,
        message: Box<ParsedExpr>,
    },
}

////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ParsedExpr {
    expr: Expr,
    span: Range<usize>,
}

////////////////////////////////////////////////////////////////
// construction / conversion
////////////////////////////////////////////////////////////////

impl ParsedExpr {
    pub fn from_kind_and_span(expr: Expr, span: Range<usize>) -> Self {
        Self { expr, span }
    }

    /// Return a new Expr from the given ExprKind and with a default span. Primariliy intended for
    /// use in testing.
    ///
    pub fn from_kind_default(expr: Expr) -> Self {
        Self {
            expr,
            span: Range::default(),
        }
    }

    /// Return a new String kind Expr with a default span. Primariliy intended for use in testing.
    ///
    pub fn from_str_default(string: &str) -> Self {
        Self {
            expr: Expr::String(string.to_string()),
            span: Range::default(),
        }
    }

    /// Return a new Uint kind Expr with a default span. Primariliy intended for use in testing.
    ///
    pub fn from_uint_default(uint: u32) -> Self {
        Self {
            expr: Expr::UInt(uint),
            span: Range::default(),
        }
    }
}

////////////////////////////////////////////////////////////////

impl From<Expr> for ExprKind {
    fn from(expr: Expr) -> Self {
        match expr {
            Expr::String(_) => ExprKind::String,
            Expr::UInt(_) => ExprKind::UInt,
            Expr::ScriptComment(_) => ExprKind::ScriptComment,
            Expr::HPMode => ExprKind::HPMode,
            Expr::Comment(_) => ExprKind::Comment,
            Expr::Wait(_) => ExprKind::Wait,
            Expr::OpenDialog(_) => ExprKind::OpenDialog,
            Expr::WaitDialog(_) => ExprKind::WaitDialog,
            Expr::Flush => ExprKind::Flush,
            Expr::Protocol => ExprKind::Protocol,
            Expr::Print(_) => ExprKind::Print,
            Expr::SetTimeFormat(_) => ExprKind::SetTimeFormat,
            Expr::SetTime => ExprKind::SetTime,
            Expr::SetOption { .. } => ExprKind::SetOption,
            Expr::TCUClose(_) => ExprKind::TCUClose,
            Expr::TCUOpen(_) => ExprKind::TCUOpen,
            Expr::TCUTest { .. } => ExprKind::TCUTest,
            Expr::PrinterSet(_) => ExprKind::PrinterSet,
            Expr::PrinterTest { .. } => ExprKind::PrinterTest,
            Expr::IssueTest(_) => ExprKind::IssueTest,
            Expr::TestResult { .. } => ExprKind::TestResult,
            Expr::USBOpen => ExprKind::USBOpen,
            Expr::USBClose => ExprKind::USBClose,
            Expr::USBPrint(_) => ExprKind::USBPrint,
            Expr::USBSetTimeFormat(_) => ExprKind::USBSetTimeFormat,
            Expr::USBSetTime => ExprKind::USBSetTime,
            Expr::USBSetOption { .. } => ExprKind::USBSetOption,
            Expr::USBPrinterSet(_) => ExprKind::USBPrinterSet,
            Expr::USBPrinterTest { .. } => ExprKind::USBPrinterTest,
        }
    }
}

////////////////////////////////////////////////////////////////

impl From<&Expr> for ExprKind {
    fn from(expr: &Expr) -> Self {
        match expr {
            Expr::String(_) => ExprKind::String,
            Expr::UInt(_) => ExprKind::UInt,
            Expr::ScriptComment(_) => ExprKind::ScriptComment,
            Expr::HPMode => ExprKind::HPMode,
            Expr::Comment(_) => ExprKind::Comment,
            Expr::Wait(_) => ExprKind::Wait,
            Expr::OpenDialog(_) => ExprKind::OpenDialog,
            Expr::WaitDialog(_) => ExprKind::WaitDialog,
            Expr::Flush => ExprKind::Flush,
            Expr::Protocol => ExprKind::Protocol,
            Expr::Print(_) => ExprKind::Print,
            Expr::SetTimeFormat(_) => ExprKind::SetTimeFormat,
            Expr::SetTime => ExprKind::SetTime,
            Expr::SetOption { .. } => ExprKind::SetOption,
            Expr::TCUClose(_) => ExprKind::TCUClose,
            Expr::TCUOpen(_) => ExprKind::TCUOpen,
            Expr::TCUTest { .. } => ExprKind::TCUTest,
            Expr::PrinterSet(_) => ExprKind::PrinterSet,
            Expr::PrinterTest { .. } => ExprKind::PrinterTest,
            Expr::IssueTest(_) => ExprKind::IssueTest,
            Expr::TestResult { .. } => ExprKind::TestResult,
            Expr::USBOpen => ExprKind::USBOpen,
            Expr::USBClose => ExprKind::USBClose,
            Expr::USBPrint(_) => ExprKind::USBPrint,
            Expr::USBSetTimeFormat(_) => ExprKind::USBSetTimeFormat,
            Expr::USBSetTime => ExprKind::USBSetTime,
            Expr::USBSetOption { .. } => ExprKind::USBSetOption,
            Expr::USBPrinterSet(_) => ExprKind::USBPrinterSet,
            Expr::USBPrinterTest { .. } => ExprKind::USBPrinterTest,
        }
    }
}

////////////////////////////////////////////////////////////////
// field access
////////////////////////////////////////////////////////////////

impl ParsedExpr {
    pub fn expression(&self) -> &Expr {
        &self.expr
    }

    pub fn span(&self) -> &Range<usize> {
        &self.span
    }
}

////////////////////////////////////////////////////////////////
// comparison
////////////////////////////////////////////////////////////////

impl std::cmp::PartialEq for ParsedExpr {
    fn eq(&self, other: &Self) -> bool {
        // Only compare the expression kind. Makes testing much easier.
        self.expr == other.expr
    }
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
}

////////////////////////////////////////////////////////////////
