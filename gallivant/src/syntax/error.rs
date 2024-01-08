use ariadne::{Label, Report, ReportKind};

use super::expression::ExprKind;

pub use crate::error::ErrorNote;

type Span = std::ops::Range<usize>;

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ErrorReason {
    Unexpected {
        span: Span,
        expected: Vec<Option<char>>,
        found: Option<char>,
    },
    Unclosed,

    /// An argument was of the wrong type.
    UnrecognisedCommand {
        span: Span,
    },

    /// An argument was of the wrong type.
    ArgType {
        span: Span,
        expected: Vec<&'static str>,
        found: &'static str,
    },

    /// An argument value beyond limits.
    ArgValue {
        span: Span,
        value: u32,
        limits: (u32, u32),
    },
}

////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    reason: ErrorReason,
    notes: Vec<ErrorNote>,
}

////////////////////////////////////////////////////////////////
// construction / conversion
////////////////////////////////////////////////////////////////

impl Error {
    pub fn unrecognised_command(span: Span) -> Self {
        Self {
            reason: ErrorReason::UnrecognisedCommand { span },
            notes: Vec::new(),
        }
    }

    /// Create a new error resulting from an argument being the wrong type.
    ///
    /// # Arguments
    /// * `span` - Area in the input that the error occured.
    /// * `expected` - Expected argument types. Only the enum variant is used here, not the values.
    /// * `found` - Type of the found argument.
    ///
    /// # TODO
    /// Having to init ExprKind variants just pass in as the expected types isn't great. Consider
    /// Having a serperate Token enum which just contains valueless variants for this purpose. Could
    /// also provide methods for access the token's string and parser.
    ///
    pub fn argument_type<Iter: IntoIterator<Item = ExprKind>>(
        span: Span,
        expected: Iter,
        found: ExprKind,
    ) -> Self {
        let expected = expected.into_iter().map(|expr| expr.name()).collect();
        let found = found.name();

        Self {
            reason: ErrorReason::ArgType {
                span,
                expected,
                found,
            },
            notes: Vec::new(),
        }
    }

    /// Create a new error resulting from an arguments value being outside of limits.
    ///
    /// # Arguments
    /// * `span` - Area in the input that the error occured.
    /// * `value` - Found argument value.
    /// * `limits` - Minumum and maximum value allowed for the argument.
    ///
    pub fn argument_value_size(span: Span, value: u32, limits: (u32, u32)) -> Self {
        debug_assert!(limits.0 <= limits.1);

        Self {
            reason: ErrorReason::ArgValue {
                span,
                value,
                limits,
            },
            notes: Vec::new(),
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

    pub fn with_note(mut self, note: ErrorNote) -> Self {
        self.notes.push(note);
        self
    }
}

////////////////////////////////////////////////////////////////
// ...
////////////////////////////////////////////////////////////////

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

////////////////////////////////////////////////////////////////

impl ErrorReason {
    pub fn message(&self) -> &'static str {
        match self {
            ErrorReason::Unexpected { .. } => "Unexpected token",
            ErrorReason::Unclosed => todo!(),
            ErrorReason::UnrecognisedCommand { .. } => "Unrecognised command found",
            ErrorReason::ArgType { .. } => "Invalid argument type",
            ErrorReason::ArgValue { .. } => "Argument value exceeds limits",
        }
    }

    pub fn labels(&self) -> Vec<Label> {
        match self {
            ErrorReason::Unexpected {
                span,
                expected,
                found,
            } => {
                let expected: Vec<String> = expected
                    .iter()
                    .map(|e| e.map_or("End of input".to_owned(), |c| c.to_string()))
                    .collect();
                let found = found.map_or("End of input".to_owned(), |c| c.to_string());

                let expected_str = if expected.len() == 1 {
                    format!("Expected '{}'", expected[0])
                } else if let Some(first) = expected.first() {
                    let expected = expected
                        .iter()
                        .skip(1)
                        .map(|s| format!("'{}'", s))
                        .fold(format!("'{}'", first), |acc, s| format!("{acc}, {s}"));

                    format!("Expected one of {}", expected)
                } else {
                    String::from("Expected none")
                };

                vec![
                    Label::new(span.clone())
                        .with_message(expected_str)
                        .with_priority(10),
                    Label::new(span.clone())
                        .with_message(format!("Found '{}'", found))
                        .with_priority(9),
                ]
            }
            ErrorReason::Unclosed => todo!(),

            ErrorReason::UnrecognisedCommand { span } => {
                vec![Label::new(span.clone())
                    .with_message("Unrecognised command")
                    .with_priority(10)]
            }

            ErrorReason::ArgType {
                span,
                expected,
                found,
            } => {
                let expected_str = if expected.len() == 1 {
                    format!("Expected '{}'", expected[0])
                } else if let Some(first) = expected.first() {
                    let expected = expected
                        .iter()
                        .skip(1)
                        .map(|s| format!("'{s}'"))
                        .fold(format!("'{first}'"), |acc, s| format!("{acc}, {s}"));

                    format!("Expected one of {}", expected)
                } else {
                    String::from("None")
                };

                vec![
                    Label::new(span.clone())
                        .with_message(expected_str)
                        .with_priority(10),
                    Label::new(span.clone())
                        .with_message(format!("Found '{}'", found))
                        .with_priority(9),
                ]
            }

            ErrorReason::ArgValue {
                span,
                value,
                limits,
            } => {
                let (min, max) = limits;
                vec![
                    Label::new(span.clone())
                        .with_message(format!("Argument has value {value}"))
                        .with_priority(10),
                    Label::new(span.clone())
                        .with_message(format!("Argument must be between {min} and {max}"))
                        .with_priority(9),
                ]
            }
        }
    }
}

////////////////////////////////////////////////////////////////

impl Error {
    pub fn to_report(&self) -> Report {
        let mut report = Report::build(ReportKind::Error, (), 0)
            .with_message(self.reason.message())
            .with_labels(self.reason.labels());

        for note in self.notes.iter() {
            report = match note {
                ErrorNote::Note(msg) => report.with_note(msg),
                ErrorNote::Help(msg) => report.with_help(msg),
            };
        }

        report.finish()
    }
}

////////////////////////////////////////////////////////////////

impl std::error::Error for Error {}

////////////////////////////////////////////////////////////////

impl chumsky::error::Error<char> for Error {
    type Span = Span;
    type Label = &'static str;

    fn expected_input_found<Iter: IntoIterator<Item = Option<char>>>(
        span: Self::Span,
        expected: Iter,
        found: Option<char>,
    ) -> Self {
        Self {
            reason: ErrorReason::Unexpected {
                span,
                expected: expected.into_iter().collect(),
                found,
            },
            notes: Vec::new(),
        }
    }

    fn with_label(mut self, label: Self::Label) -> Self {
        self.notes.push(ErrorNote::Note(label));
        self
    }

    fn merge(self, _: Self) -> Self {
        self
    }
}

////////////////////////////////////////////////////////////////
