use std::ops::Range;

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

#[derive(PartialEq, Clone, Debug)]
pub enum ExprKind {
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
// comparison
////////////////////////////////////////////////////////////////

impl std::cmp::PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        // Only compare the expression kind. Makes testing much easier.
        self.kind == other.kind
    }
}

////////////////////////////////////////////////////////////////
