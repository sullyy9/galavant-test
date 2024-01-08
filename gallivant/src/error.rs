use ariadne::{Config, Label, Report, ReportKind};

use crate::{
    execution::FailedTest,
    syntax::{self, Expr, ParsedExpr},
};

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Error {
    reason: ErrorReason,
    notes: Vec<ErrorNote>,
}

////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ErrorReason {
    SyntaxError(syntax::ErrorReason),
    TestFailure {
        expression: ParsedExpr,
        test: FailedTest,
    },
    IOError {
        expression: ParsedExpr,
        error: std::io::Error,
    },
}

////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorNote {
    Note(&'static str),
    Help(&'static str),
}

////////////////////////////////////////////////////////////////
// construction / conversion
////////////////////////////////////////////////////////////////

impl Error {
    pub fn from_io_error(expression: ParsedExpr, error: std::io::Error) -> Self {
        Self {
            reason: ErrorReason::IOError { expression, error },
            notes: Vec::new(),
        }
    }

    pub fn from_failed_test(expression: ParsedExpr, test: FailedTest) -> Self {
        Self {
            reason: ErrorReason::TestFailure { expression, test },
            notes: Vec::new(),
        }
    }

    pub fn with_note(mut self, note: ErrorNote) -> Self {
        self.notes.push(note);
        self
    }
}

////////////////////////////////////////////////////////////////

impl From<syntax::Error> for Error {
    fn from(error: syntax::Error) -> Self {
        Self {
            reason: ErrorReason::SyntaxError(error.reason().to_owned()),
            notes: error.notes().to_owned(),
        }
    }
}

////////////////////////////////////////////////////////////////

impl From<Error> for Report<'_> {
    fn from(error: Error) -> Self {
        Report::from(&error)
    }
}

////////////////////////////////////////////////////////////////

impl From<&Error> for Report<'_> {
    fn from(error: &Error) -> Self {
        let mut report = Report::build(ReportKind::Error, (), 0)
            .with_config(Config::default().with_cross_gap(true))
            .with_message(error.reason.message())
            .with_labels(error.reason.labels());

        for note in error.notes.iter() {
            report = match note {
                ErrorNote::Note(msg) => report.with_note(msg),
                ErrorNote::Help(msg) => report.with_help(msg),
            };
        }

        report.finish()
    }
}

////////////////////////////////////////////////////////////////

impl ErrorReason {
    pub fn message(&self) -> String {
        match self {
            ErrorReason::SyntaxError(reason) => format!("Syntax error - {}", reason.message()),
            ErrorReason::TestFailure { test, .. } => format!("Test failed - {}", test.message),
            ErrorReason::IOError { error, .. } => format!("IO error - {}", error),
        }
    }

    pub fn labels(&self) -> Vec<Label> {
        match self {
            ErrorReason::SyntaxError(reason) => reason.labels(),

            ErrorReason::TestFailure { expression, test } => {
                let range_expr = match expression.expression() {
                    Expr::TCUTest { min, max, .. } => Some((min, max)),
                    Expr::PrinterTest { min, max, .. } => Some((min, max)),
                    Expr::USBPrinterTest { min, max, .. } => Some((min, max)),
                    _ => None,
                };

                // Create a label highlighting the failing command.
                let mut labels = Vec::new();

                // Create a label highlighting the bound that the measured value violated.
                if test.measurement > *test.expected.end() {
                    let span = range_expr
                        .map(|(_, max)| max.span())
                        .unwrap_or(expression.span());

                    labels.push(
                        Label::new(span.clone())
                            .with_message(format!(
                                "Expected maximum value of {} but measured {}",
                                test.expected.end(),
                                test.measurement
                            ))
                            .with_order(1),
                    );
                }

                if test.measurement < *test.expected.start() {
                    let span = range_expr
                        .map(|(min, _)| min.span())
                        .unwrap_or(expression.span());

                    labels.push(Label::new(span.clone()).with_message(format!(
                        "Expected minimum value of {} but measured {}",
                        test.expected.start(),
                        test.measurement
                    )));
                }

                labels
            }

            ErrorReason::IOError { expression, .. } => {
                vec![Label::new(expression.span().clone())
                    .with_message("When executing this command")]
            }
        }
    }
}

////////////////////////////////////////////////////////////////
// field access
////////////////////////////////////////////////////////////////

impl Error {
    pub fn reason(&self) -> &ErrorReason {
        &self.reason
    }

    pub fn notes(&self) -> &[ErrorNote] {
        &self.notes
    }
}

////////////////////////////////////////////////////////////////
// ...
////////////////////////////////////////////////////////////////

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Report::from(self))
    }
}

////////////////////////////////////////////////////////////////

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.reason {
            ErrorReason::SyntaxError(_) => None,
            ErrorReason::TestFailure { .. } => None,
            ErrorReason::IOError {
                expression: _,
                error,
            } => Some(error),
        }
    }
}

////////////////////////////////////////////////////////////////
