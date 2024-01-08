mod mocks;

use std::time::Duration;

use gallivant::{Dialog, Error, FrontendRequest, Interpreter, TransactionStatus};

type Request = FrontendRequest;

use mocks::PortMock;

////////////////////////////////////////////////////////////////

#[test]
fn test_hpmode() {
    let script = r#"HPMODE"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0], Request::None)
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_comment() {
    let script = r#"COMMENT "This is a comment 1234""#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(
                requests[0],
                Request::GuiPrint(String::from("This is a comment 1234"))
            )
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_wait() {
    let script = r#"WAIT 12345"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0], Request::Wait(Duration::from_millis(12345)))
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_opendialog() {
    let script = r#"OPENDIALOG "Open a dialog""#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(
                requests[0],
                Request::GuiDialogue {
                    kind: Dialog::Notification,
                    message: String::from("Open a dialog")
                }
            )
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_waitdialog() {
    let script = r#"WAITDIALOG "Open a wait dialog""#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(
                requests[0],
                Request::GuiDialogue {
                    kind: Dialog::ManualInput,
                    message: String::from("Open a wait dialog")
                }
            )
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_flush() {
    let script = r#"FLUSH"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0], Request::TCUFlush)
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_protocol() {
    let script = r#"PROTOCOL"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0], Request::None)
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_print() {
    let script = r#"PRINT "t", 123, $F3"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Request::TCUTransact(_)));

            if let Request::TCUTransact(mut transaction) = request {
                let mut port = PortMock::new();

                if let Ok(TransactionStatus::Ongoing(tr)) = transaction.process(&mut port) {
                    assert_eq!(port.txdata, b"P06747BF3\r");
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
fn test_settimeformat() {
    let script = r#"SETTIMEFORMAT 5"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Request::TCUTransact(_)));

            if let Request::TCUTransact(mut transaction) = request {
                let mut port = PortMock::new();

                if let Ok(TransactionStatus::Ongoing(tr)) = transaction.process(&mut port) {
                    assert_eq!(port.txdata, b"P051B746605\r");
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
fn test_setoption() {
    let script = r#"SETOPTION 6, 8"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Request::TCUTransact(_)));

            if let Request::TCUTransact(mut transaction) = request {
                let mut port = PortMock::new();

                if let Ok(TransactionStatus::Ongoing(tr)) = transaction.process(&mut port) {
                    assert_eq!(port.txdata, b"P061B004F0608\r");
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
fn test_tcuclose() {
    let script = r#"TCUCLOSE 6"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Request::TCUTransact(_)));

            if let Request::TCUTransact(mut transaction) = request {
                let mut port = PortMock::new();

                if let Ok(TransactionStatus::Ongoing(tr)) = transaction.process(&mut port) {
                    assert_eq!(port.txdata, b"C06\r");
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
fn test_tcuopen() {
    let script = r#"TCUOPEN 2"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Request::TCUTransact(_)));

            if let Request::TCUTransact(mut transaction) = request {
                let mut port = PortMock::new();

                if let Ok(TransactionStatus::Ongoing(tr)) = transaction.process(&mut port) {
                    assert_eq!(port.txdata, b"O02\r");
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
fn test_tcutest() {
    let script = r#"TCUTEST 3, 1000, 12000, 1, "FAIL""#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Request::TCUTransact(_)));

            if let Request::TCUTransact(mut transaction) = request {
                let mut port = PortMock::new();

                if let Ok(TransactionStatus::Ongoing(tr)) = transaction.process(&mut port) {
                    assert_eq!(port.txdata, b"M03\r");
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
fn test_printerset() {
    let script = r#"PRINTERSET 2"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Request::TCUTransact(_)));

            if let Request::TCUTransact(mut transaction) = request {
                let mut port = PortMock::new();

                if let Ok(TransactionStatus::Ongoing(tr)) = transaction.process(&mut port) {
                    assert_eq!(port.txdata, b"P051B005302\r");
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
fn test_printertest() {
    let script = r#"PRINTERTEST 3, 1000, 12000, 1, "FAIL""#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Request::TCUTransact(_)));

            if let Request::TCUTransact(mut transaction) = request {
                let mut port = PortMock::new();

                if let Ok(TransactionStatus::Ongoing(tr)) = transaction.process(&mut port) {
                    assert_eq!(port.txdata, b"W051B004D03\r");
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
fn test_usbopen() {
    let script = r#"USBOPEN"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0], Request::PrinterOpen)
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_usbclose() {
    let script = r#"USBCLOSE"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0], Request::PrinterClose)
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_usbprint() {
    let script = r#"USBPRINT "test", 45, $D4"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            let mut expected = "test".as_bytes().to_owned();
            expected.extend_from_slice(&[45, 0xD4]);

            if let Request::PrinterTransact(transaction) = requests[0].clone() {
                let mut port = PortMock::new();
                assert_eq!(
                    transaction.process(&mut port).unwrap(),
                    TransactionStatus::Success
                );

                assert_eq!(port.txdata, expected)
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
fn test_usbsettimeformat() {
    let script = r#"USBSETTIMEFORMAT 6"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);

            if let Request::PrinterTransact(transaction) = requests[0].clone() {
                let mut port = PortMock::new();
                assert_eq!(
                    transaction.process(&mut port).unwrap(),
                    TransactionStatus::Success
                );

                assert_eq!(port.txdata, vec![0x1B, b't', b'f', 6])
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
fn test_usbsetoption() {
    let script = r#"USBSETOPTION 6, 7"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);

            if let Request::PrinterTransact(transaction) = requests[0].clone() {
                let mut port = PortMock::new();
                assert_eq!(
                    transaction.process(&mut port).unwrap(),
                    TransactionStatus::Success
                );

                assert_eq!(port.txdata, vec![0x1B, 0x00, b'O', 6, 7])
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
fn test_usbprinterset() {
    let script = r#"USBPRINTERSET 2"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);

            if let Request::PrinterTransact(transaction) = requests[0].clone() {
                let mut port = PortMock::new();
                assert_eq!(
                    transaction.process(&mut port).unwrap(),
                    TransactionStatus::Success
                );

                assert_eq!(port.txdata, vec![0x1B, 0x00, b'S', 2])
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
fn test_usbprintertest() {
    let script = r#"USBPRINTERTEST 3, 1000, 12000, 1, "FAIL""#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();
            let requests: Vec<FrontendRequest> = requests.into_iter().map(|r| r.unwrap()).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Request::PrinterTransact(_)));

            if let Request::TCUTransact(mut transaction) = request {
                let mut port = PortMock::new();

                if let Ok(TransactionStatus::Ongoing(tr)) = transaction.process(&mut port) {
                    assert_eq!(port.txdata, vec![0x1B, 0x00, b'M', 3]);
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
