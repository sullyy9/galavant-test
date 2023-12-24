mod args;
mod mock;
mod port;

use std::io::Write;

use ariadne::Source;
use clap::Parser;
use serialport::{self, SerialPort};

use galavant::{FrontendRequest, Interpreter};

use self::{args::Args, mock::MockTCUPort, port::CommPort};

////////////////////////////////////////////////////////////////

enum Error {
    ParseErrors(Vec<galavant::Error>),
    RuntimeError(galavant::Error),
}

impl From<Vec<galavant::Error>> for Error {
    fn from(errors: Vec<galavant::Error>) -> Self {
        Self::ParseErrors(errors)
    }
}

impl From<galavant::Error> for Error {
    fn from(error: galavant::Error) -> Self {
        Self::RuntimeError(error)
    }
}

////////////////////////////////////////////////////////////////

fn main() {
    let args = Args::parse();

    let mut tcu: Option<Box<dyn SerialPort>> = match args.tcu {
        Some(port) if port == "mock" => Some(Box::new(MockTCUPort::new())),
        Some(port) => Some(
            serialport::new(port, 9600)
                .open()
                .expect("Failed to open TCU port"),
        ),
        None => None,
    };

    let mut printer = args
        .printer
        .map(|port| CommPort::from(serialport::new(port, 9600)));

    let script = std::fs::read_to_string(&args.script).expect("Failed to read script");

    let run_script = |i| run_script(i, args.debug, &mut tcu, &mut printer);

    match galavant::Interpreter::try_from_str(&script)
        .map_err(Error::from)
        .and_then(run_script)
    {
        Ok(()) => (),
        Err(Error::ParseErrors(errors)) => {
            for error in errors {
                error
                    .to_report()
                    .eprint(Source::from(&script))
                    .expect("Failed to create error report");
            }
        }
        Err(Error::RuntimeError(_)) => todo!(),
    }
}

////////////////////////////////////////////////////////////////

fn run_script(
    interpreter: Interpreter,
    debug: bool,
    tcu: &mut Option<Box<dyn SerialPort>>,
    printer: &mut Option<CommPort>,
) -> Result<(), Error> {
    for current_request in interpreter {
        let mut current_request = Some(current_request?);

        while let Some(request) = current_request {
            current_request = handle_request(request, debug, tcu, printer);
        }
    }

    Ok(())
}

////////////////////////////////////////////////////////////////

fn handle_request(
    request: FrontendRequest,
    debug: bool,
    tcu: &mut Option<Box<dyn SerialPort>>,
    printer: &mut Option<CommPort>,
) -> Option<FrontendRequest> {
    if debug {
        println!("{request:?}")
    }

    match request {
        FrontendRequest::None => (),
        FrontendRequest::Wait(time) => std::thread::sleep(time),

        FrontendRequest::GuiPrint(message) => println!("COMMENT: {message}"),
        FrontendRequest::GuiDialogue { kind, message } => match kind {
            galavant::Dialog::ManualInput => {
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
            galavant::Dialog::Notification => println!("DIALOG:  {message}"),
        },

        FrontendRequest::TCUTransact(transaction) => {
            if let Some(tcu) = tcu {
                tcu.write_all(transaction.bytes())
                    .expect("TCU transmit error");
                return Some(FrontendRequest::TCUAwaitResponse(transaction));
            } else {
                panic!("TCU port required but none given");
            }
        }

        FrontendRequest::TCUAwaitResponse(transaction) => {
            if let Some(tcu) = tcu {
                let response = tcu
                    .bytes_to_read()
                    .map(|bytes| {
                        let mut response = vec![0; bytes as usize];
                        tcu.read_exact(&mut response).expect("TCU receive error");
                        response
                    })
                    .expect("TCU receive error");

                match transaction.evaluate(&response) {
                    Ok(request) => return Some(request),
                    Err(_) => todo!(),
                }
            } else {
                panic!("TCU port required but none given");
            }
        }

        FrontendRequest::TCUFlush => {
            if let Some(tcu) = tcu {
                tcu.flush().expect("TCU transmit error");
                let mut buffer = Vec::new();
                tcu.read_to_end(&mut buffer).expect("TCU receive error");
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
                port.close().expect("Failed to open printer comm port");
            } else {
                panic!("Printer port required but none given");
            }
        }

        FrontendRequest::PrinterTransmit(tx) => match printer {
            Some(CommPort::Open(port)) => port.write_all(&tx).expect("Printer transmit error"),

            Some(CommPort::Closed(_)) => {
                panic!("Attempted to write to printer comm port but port is not open")
            }
            None => panic!("Printer port required but none given"),
        },

        FrontendRequest::PrinterTransact(transaction) => match printer {
            Some(CommPort::Open(port)) => {
                port.write_all(transaction.bytes())
                    .expect("Printer transmit error");
                return Some(FrontendRequest::PrinterAwaitResponse(transaction));
            }

            Some(CommPort::Closed(_)) => {
                panic!("Attempted to write to printer comm port but port is not open")
            }
            None => panic!("Printer port required but none given"),
        },
        FrontendRequest::PrinterAwaitResponse(transaction) => match printer {
            Some(CommPort::Open(port)) => {
                let response = port
                    .bytes_to_read()
                    .map(|bytes| {
                        let mut response = vec![0; bytes as usize];
                        port.read_exact(&mut response)
                            .expect("Printer receive error");
                        response
                    })
                    .expect("Printer receive error");

                match transaction.evaluate(&response) {
                    Ok(request) => return Some(request),
                    Err(_) => todo!(),
                }
            }

            Some(CommPort::Closed(_)) => {
                panic!("Attempted to write to printer comm port but port is not open")
            }
            None => panic!("Printer port required but none given"),
        },
    }

    None
}

////////////////////////////////////////////////////////////////
