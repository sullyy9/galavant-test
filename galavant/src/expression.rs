use std::ops::Range;

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

#[derive(PartialEq, Clone, Debug)]
pub enum ExprKind {
    String(String),
    UInt(u32),

    /// All this command ever did was add a NULL after ESC when sending the ESC commands for the
    /// following test commands:
    /// SETTIMEFORMAT
    /// SETTIME
    /// SETOPTION
    /// PRINTERSET
    /// PRINTERTEST
    /// and their USB variants.
    ///
    /// This was in order to direct the ESC command to the printer's debug protocol handler.
    /// Extra NULL's are ignored by the debug protocol i.e. ESC NULL NULL NULL 's' is handled
    /// the same as ESC NULL 's'. Therefore, rather than keeping an internal state here, we can just
    /// always send the extra NULL for those commands. Essentially this command will be always
    /// on and using it will not do anything.
    ///
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

#[derive(Clone, Debug)]
pub struct Expr {
    kind: ExprKind,
    span: Range<usize>,
}

////////////////////////////////////////////////////////////////
// construction / conversion
////////////////////////////////////////////////////////////////

impl Expr {
    pub fn from_kind_and_span(kind: ExprKind, span: Range<usize>) -> Self {
        Self { kind, span }
    }

    /// Return a new Expr from the given ExprKind and with a default span. Primariliy intended for
    /// use in testing.
    ///
    pub fn from_kind_default(kind: ExprKind) -> Self {
        Self {
            kind,
            span: Range::default(),
        }
    }

    /// Return a new String kind Expr with a default span. Primariliy intended for use in testing.
    ///
    pub fn from_str_default(string: &str) -> Self {
        Self {
            kind: ExprKind::String(string.to_string()),
            span: Range::default(),
        }
    }

    /// Return a new Uint kind Expr with a default span. Primariliy intended for use in testing.
    ///
    pub fn from_uint_default(uint: u32) -> Self {
        Self {
            kind: ExprKind::UInt(uint),
            span: Range::default(),
        }
    }
}

////////////////////////////////////////////////////////////////
// field access
////////////////////////////////////////////////////////////////

impl Expr {
    pub fn kind(&self) -> &ExprKind {
        &self.kind
    }

    pub fn span(&self) -> &Range<usize> {
        &self.span
    }
}

////////////////////////////////////////////////////////////////
// comparison
////////////////////////////////////////////////////////////////

impl std::cmp::PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        // Only compare the expression kind. Makes testing much easier.
        self.kind == other.kind
    }
}

////////////////////////////////////////////////////////////////

impl ExprKind {
    pub fn kind_name(&self) -> &'static str {
        match self {
            ExprKind::String(_) => "String",
            ExprKind::UInt(_) => "Unsigned Integer",
            ExprKind::HPMode => "Command: 'HPMODE'",
            ExprKind::Comment(_) => "Command: 'COMMENT'",
            ExprKind::Wait(_) => "Command: 'WAIT'",
            ExprKind::OpenDialog(_) => "Command: 'OPENDIALOG'",
            ExprKind::WaitDialog(_) => "Command: 'WAITDIALOG'",
            ExprKind::Flush => "Command: 'FLUSH'",
            ExprKind::Protocol => "Command: 'PROTOCOL'",
            ExprKind::Print(_) => "Command: 'PRINT'",
            ExprKind::SetTimeFormat(_) => "Command: 'SETTIMEFORMAT'",
            ExprKind::SetTime => "Command: 'SETTIME'",
            ExprKind::SetOption { .. } => "Command: 'SETOPTION'",
            ExprKind::TCUClose(_) => "Command: 'TCUCLOSE'",
            ExprKind::TCUOpen(_) => "Command: 'TCUOPEN'",
            ExprKind::TCUTest { .. } => "Command: 'TCUTEST'",
            ExprKind::PrinterSet(_) => "Command: 'PRINTERSET'",
            ExprKind::PrinterTest { .. } => "Command: 'PRINTERTEST'",
            ExprKind::IssueTest(_) => "Command: 'ISSUETEST'",
            ExprKind::TestResult { .. } => "Command: 'TESTRESULT'",
            ExprKind::USBOpen => "Command: 'USBOPEN'",
            ExprKind::USBClose => "Command: 'USBCLOSE'",
            ExprKind::USBPrint(_) => "Command: 'USBPRINT'",
            ExprKind::USBSetTimeFormat(_) => "Command: 'USBSETTIMEFORMAT'",
            ExprKind::USBSetTime => "Command: 'USBSETTIME'",
            ExprKind::USBSetOption { .. } => "Command: 'USBSETOPTION'",
            ExprKind::USBPrinterSet(_) => "Command: 'USBPRINTERSET'",
            ExprKind::USBPrinterTest { .. } => "Command: 'USBPRINTERTEST'",
        }
    }
}

////////////////////////////////////////////////////////////////
