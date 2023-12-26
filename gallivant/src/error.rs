use ariadne::{Label, Report, ReportKind};

use crate::expression::ExprKind;

type Span = std::ops::Range<usize>;

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Reason {
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Error {
    reason: Reason,
    notes: Vec<&'static str>,
    help: Vec<&'static str>,
}

////////////////////////////////////////////////////////////////
// construction / conversion
////////////////////////////////////////////////////////////////

impl Error {
    pub fn unrecognised_command(span: Span) -> Self {
        Self {
            reason: Reason::UnrecognisedCommand { span },
            notes: Vec::new(),
            help: Vec::new(),
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
    pub fn argument_type<'a, Iter: IntoIterator<Item = &'a ExprKind>>(
        span: Span,
        expected: Iter,
        found: &ExprKind,
    ) -> Self {
        let expected = expected.into_iter().map(|expr| expr.kind_name()).collect();
        let found = found.kind_name();

        Self {
            reason: Reason::ArgType {
                span,
                expected,
                found,
            },
            notes: Vec::new(),
            help: Vec::new(),
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
            reason: Reason::ArgValue {
                span,
                value,
                limits,
            },
            notes: Vec::new(),
            help: Vec::new(),
        }
    }
}

////////////////////////////////////////////////////////////////
// field access
////////////////////////////////////////////////////////////////

impl Error {
    pub fn reason(&self) -> &Reason {
        &self.reason
    }

    pub fn notes(&self) -> &[&'static str] {
        &self.notes
    }

    pub fn help(&self) -> &[&'static str] {
        &self.help
    }

    pub fn with_note(mut self, note: &'static str) -> Self {
        self.notes.push(note);
        self
    }

    pub fn with_help(mut self, help: &'static str) -> Self {
        self.help.push(help);
        self
    }
}

////////////////////////////////////////////////////////////////
// ...
////////////////////////////////////////////////////////////////

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_report())
    }
}

////////////////////////////////////////////////////////////////

impl Reason {
    fn as_message(&self) -> &'static str {
        match self {
            Reason::Unexpected { .. } => "Unexpected token",
            Reason::Unclosed => todo!(),
            Reason::UnrecognisedCommand { .. } => "Unrecognised command found",
            Reason::ArgType { .. } => "Invalid argument type",
            Reason::ArgValue { .. } => "Argument value exceeds limits",
        }
    }

    fn labels(&self) -> Vec<Label> {
        match self {
            Reason::Unexpected {
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
            Reason::Unclosed => todo!(),

            Reason::UnrecognisedCommand { span } => {
                vec![Label::new(span.clone())
                    .with_message("Unrecognised command")
                    .with_priority(10)]
            }

            Reason::ArgType {
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

            Reason::ArgValue {
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
            .with_message(self.reason.as_message())
            .with_labels(self.reason.labels());

        for note in self.notes.iter() {
            report = report.with_note(note);
        }

        for help in self.help.iter() {
            report = report.with_help(help);
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
            reason: Reason::Unexpected {
                span,
                expected: expected.into_iter().collect(),
                found,
            },
            notes: Vec::new(),
            help: Vec::new(),
        }
    }

    fn with_label(mut self, label: Self::Label) -> Self {
        self.notes.push(label);
        self
    }

    fn merge(self, _: Self) -> Self {
        self
    }
}

////////////////////////////////////////////////////////////////
