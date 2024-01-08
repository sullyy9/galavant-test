mod mocks;

use gallivant::{Error, FrontendRequest, Interpreter, TransactionStatus};

type Request = FrontendRequest;

use mocks::PortMock;

////////////////////////////////////////////////////////////////

#[test]
fn test_hpmode_settimeformat() {
    let script = r#"
HPMODE
SETTIMEFORMAT 5
    "#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();

            assert_eq!(requests.len(), 2);
            let request = requests[1].as_ref().unwrap().clone();

            assert!(matches!(request, Request::TCUTransact(_)));

            if let Request::TCUTransact(mut transaction) = request {
                let mut port = PortMock::new();

                if let Ok(TransactionStatus::Ongoing(tr)) = transaction.process(&mut port) {
                    assert_eq!(port.txdata, b"P051B00746605\r");
                    transaction = tr;
                } else {
                    panic!()
                }

                // Echo.
                port.rxdata.extend(&port.txdata);
                assert!(matches!(
                    transaction.process(&mut port),
                    Ok(TransactionStatus::Success)
                ));
            }
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_hpmode_setoption() {
    let script = r#"
HPMODE
SETOPTION 6, 8
    "#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();

            assert_eq!(requests.len(), 2);
            let request = requests[1].as_ref().unwrap().clone();

            assert!(matches!(request, Request::TCUTransact(_)));

            if let Request::TCUTransact(mut transaction) = request {
                let mut port = PortMock::new();

                if let Ok(TransactionStatus::Ongoing(tr)) = transaction.process(&mut port) {
                    assert_eq!(port.txdata, b"P061B00004F0608\r");
                    transaction = tr;
                } else {
                    panic!()
                }

                // Echo.
                port.rxdata.extend(&port.txdata);
                assert!(matches!(
                    transaction.process(&mut port),
                    Ok(TransactionStatus::Success)
                ));
            }
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_hpmode_printerset() {
    let script = r#"
HPMODE    
PRINTERSET 2
    "#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();

            assert_eq!(requests.len(), 2);
            let request = requests[1].as_ref().unwrap().clone();

            assert!(matches!(request, Request::TCUTransact(_)));

            if let Request::TCUTransact(mut transaction) = request {
                let mut port = PortMock::new();

                if let Ok(TransactionStatus::Ongoing(tr)) = transaction.process(&mut port) {
                    assert_eq!(port.txdata, b"P051B00005302\r");
                    transaction = tr;
                } else {
                    panic!()
                }

                // Echo.
                port.rxdata.extend(port.txdata.iter());
                assert_eq!(
                    transaction.process(&mut port).unwrap(),
                    TransactionStatus::Success
                );
            }
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_hpmode_printertest() {
    let script = r#"
HPMODE
PRINTERTEST 3, 1000, 12000, 1, "FAIL"
"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();

            assert_eq!(requests.len(), 2);
            let request = requests[1].as_ref().unwrap().clone();

            assert!(matches!(request, Request::TCUTransact(_)));

            if let Request::TCUTransact(mut transaction) = request {
                let mut port = PortMock::new();

                if let Ok(TransactionStatus::Ongoing(tr)) = transaction.process(&mut port) {
                    assert_eq!(port.txdata, b"W051B00004D03\r");
                    transaction = tr;
                } else {
                    panic!()
                }

                // Echo.
                port.rxdata.extend(&port.txdata);
                let result = transaction.process(&mut port);
                assert!(matches!(result, Ok(TransactionStatus::Ongoing(_))));

                // Measurement.
                if let Ok(TransactionStatus::Ongoing(tr)) = result {
                    port.rxdata.extend("AA1\r".as_bytes());
                    assert!(matches!(
                        tr.process(&mut port),
                        Ok(TransactionStatus::Success)
                    ))
                }
            }
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_hpmode_usbsettimeformat() {
    let script = r#"
HPMODE
USBSETTIMEFORMAT 6
    "#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();

            assert_eq!(requests.len(), 2);

            if let Request::PrinterTransact(transaction) = requests[1].as_ref().unwrap().clone() {
                let mut port = PortMock::new();
                assert_eq!(
                    transaction.process(&mut port).unwrap(),
                    TransactionStatus::Success
                );

                assert_eq!(port.txdata, vec![0x1B, 0x00, b't', b'f', 6])
            } else {
                panic!(
                    "Expected Request::PrinterTransact but found {:?}",
                    requests[1]
                )
            }
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_hpmode_usbsetoption() {
    let script = r#"
HPMODE
USBSETOPTION 6, 7
    "#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();

            assert_eq!(requests.len(), 2);

            if let Request::PrinterTransact(transaction) = requests[1].as_ref().unwrap().clone() {
                let mut port = PortMock::new();
                assert_eq!(
                    transaction.process(&mut port).unwrap(),
                    TransactionStatus::Success
                );

                assert_eq!(port.txdata, vec![0x1B, 0x00, 0x00, b'O', 6, 7])
            } else {
                panic!(
                    "Expected Request::PrinterTransact but found {:?}",
                    requests[1]
                )
            }
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_hpmode_usbprinterset() {
    let script = r#"
HPMODE
USBPRINTERSET 2
    "#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();

            assert_eq!(requests.len(), 2);

            if let Request::PrinterTransact(transaction) = requests[1].as_ref().unwrap().clone() {
                let mut port = PortMock::new();
                assert_eq!(
                    transaction.process(&mut port).unwrap(),
                    TransactionStatus::Success
                );

                assert_eq!(port.txdata, vec![0x1B, 0x00, 0x00, b'S', 2])
            } else {
                panic!(
                    "Expected Request::PrinterTransact but found {:?}",
                    requests[1]
                )
            }
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_hpmode_usbprintertest() {
    let script = r#"
HPMODE
USBPRINTERTEST 3, 1000, 12000, 1, "FAIL"
    "#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();

            assert_eq!(requests.len(), 2);
            let request = requests[1].as_ref().unwrap().clone();

            assert!(matches!(request, Request::PrinterTransact(_)));

            if let Request::PrinterTransact(mut transaction) = request {
                let mut port = PortMock::new();

                if let Ok(TransactionStatus::Ongoing(tr)) = transaction.process(&mut port) {
                    assert_eq!(port.txdata, vec![0x1B, 0x00, 0x00, b'M', 3]);
                    transaction = tr;
                } else {
                    panic!()
                }

                // Measurement.
                port.rxdata.extend("AA1\r".as_bytes());
                assert!(matches!(
                    transaction.process(&mut port),
                    Ok(TransactionStatus::Success)
                ));
            }
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////
