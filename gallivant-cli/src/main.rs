use std::{
    io::{ErrorKind, Write},
    time::Duration,
};

use ariadne::{Report, Source};
use clap::Parser;
use serialport::{self, SerialPort};

use gallivant::{FrontendRequest, Interpreter, Transaction, TransactionStatus};
use gallivant_serial::{CommPort, MockTCUPort};

mod args;
use args::Args;

////////////////////////////////////////////////////////////////

enum Error {
    ParseErrors(Vec<gallivant::Error>),
    RuntimeError(gallivant::Error),
}

impl From<Vec<gallivant::Error>> for Error {
    fn from(errors: Vec<gallivant::Error>) -> Self {
        Self::ParseErrors(errors)
    }
}

impl From<gallivant::Error> for Error {
    fn from(error: gallivant::Error) -> Self {
        Self::RuntimeError(error)
    }
}

////////////////////////////////////////////////////////////////

fn main() {
    let args = Args::parse();

    let mut tcu = args.tcu.map(|port| {
        if port == "mock" {
            CommPort::Open(Box::new(MockTCUPort::new()))
        } else {
            CommPort::from(
                serialport::new(port, 9600)
                    .timeout(Duration::from_millis(100))
                    .open()
                    .expect("Failed to open TCU port"),
            )
        }
    });

    let mut printer = args.printer.map(|port| {
        CommPort::from(serialport::new(port, 9600).timeout(Duration::from_millis(100)))
    });

    let script = std::fs::read_to_string(&args.script).expect("Failed to read script");

    let run_script = |i| run_script(i, args.debug, &mut tcu, &mut printer);

    match gallivant::Interpreter::try_from_str(&script)
        .map_err(Error::from)
        .and_then(run_script)
    {
        Ok(()) => (),
        Err(Error::ParseErrors(errors)) => {
            for error in errors {
                Report::from(error)
                    .eprint(Source::from(&script))
                    .expect("Failed to create error report");
            }
        }
        Err(Error::RuntimeError(error)) => {
            Report::from(error)
                .eprint(Source::from(&script))
                .expect("Failed to create error report");
        }
    }
}

////////////////////////////////////////////////////////////////

fn run_script(
    interpreter: Interpreter,
    debug: bool,
    tcu: &mut Option<CommPort>,
    printer: &mut Option<CommPort>,
) -> Result<(), Error> {
    for current_request in interpreter {
        let mut current_request = Some(current_request?);

        while let Some(request) = current_request {
            current_request = handle_request(request, debug, tcu, printer)?;
        }
    }

    Ok(())
}

////////////////////////////////////////////////////////////////

fn handle_request(
    request: FrontendRequest,
    debug: bool,
    tcu: &mut Option<CommPort>,
    printer: &mut Option<CommPort>,
) -> Result<Option<FrontendRequest>, Error> {
    if debug {
        println!("{request:?}")
    }

    match request {
        FrontendRequest::None => (),
        FrontendRequest::Wait(time) => std::thread::sleep(time),

        FrontendRequest::GuiPrint(message) => println!("COMMENT: {message}"),
        FrontendRequest::GuiDialogue { kind, message } => match kind {
            gallivant::Dialog::ManualInput => {
                println!("DIALOG:  {message}");

                loop {
                    print!("INPUT:   ");
                    std::io::stdout().flush().expect("std out flush error");

                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .expect("Dialog input error");

                    let input = input.trim();
                    if input.starts_with("STOP") || input.starts_with(['S', 's']) {
                        panic!("Test cancelled")
                    }

                    if input.starts_with("CONTINUE")
                        || input.starts_with(['C', 'c'])
                        || input.is_empty()
                    {
                        break;
                    }
                }
            }
            gallivant::Dialog::Notification => println!("DIALOG:  {message}"),
        },

        FrontendRequest::TCUTransact(transaction) => {
            if let Some(CommPort::Open(tcu)) = tcu {
                handle_transaction(transaction, tcu)?;
            } else {
                panic!("TCU port required but none given");
            }
        }

        FrontendRequest::TCUFlush => {
            if let Some(CommPort::Open(tcu)) = tcu {
                tcu.flush().expect("TCU transmit error");
                let mut buffer = Vec::new();
                match tcu.read_to_end(&mut buffer) {
                    Ok(_) => (),
                    Err(error) => match error.kind() {
                        ErrorKind::TimedOut => (),
                        _ => panic!("TCU receive error"),
                    },
                };
            } else {
                panic!("TCU port required but none given");
            }
        }

        FrontendRequest::PrinterOpen => {
            if let Some(port) = printer {
                port.open().expect("Failed to open printer comm port");
            } else {
                panic!("Printer port required but none given");
            }
        }

        FrontendRequest::PrinterClose => {
            if let Some(port) = printer {
                port.close().expect("Failed to close printer comm port");
            } else {
                panic!("Printer port required but none given");
            }
        }

        FrontendRequest::PrinterTransact(transaction) => match printer {
            Some(CommPort::Open(port)) => {
                handle_transaction(transaction, port)?;
            }

            Some(CommPort::Closed(_)) => {
                panic!("Attempted to write to printer comm port but port is not open")
            }
            None => panic!("Printer port required but none given"),
        },
    }

    Ok(None)
}

////////////////////////////////////////////////////////////////

fn handle_transaction(
    mut transaction: Transaction,
    port: &mut Box<dyn SerialPort>,
) -> Result<(), Error> {
    // Send bytes.
    loop {
        transaction = match transaction.process(port)? {
            TransactionStatus::Success => break,
            TransactionStatus::Ongoing(transaction) => transaction,
        }
    }

    Ok(())
}

////////////////////////////////////////////////////////////////
