use std::time::Duration;

use galavant::{Dialog, Error, FrontendRequest};

type Request = FrontendRequest;

////////////////////////////////////////////////////////////////

#[test]
fn test_hpmode() {
    let script = r#"HPMODE"#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0], Ok(Request::None))
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_comment() {
    let script = r#"COMMENT "This is a comment 1234""#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(
                requests[0],
                Ok(Request::GuiPrint(String::from("This is a comment 1234")))
            )
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_wait() {
    let script = r#"WAIT 12345"#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0], Ok(Request::Wait(Duration::from_millis(12345))))
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_opendialog() {
    let script = r#"OPENDIALOG "Open a dialog""#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(
                requests[0],
                Ok(Request::GuiDialogue {
                    kind: Dialog::Notification { await_close: false },
                    message: String::from("Open a dialog")
                })
            )
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_waitdialog() {
    let script = r#"WAITDIALOG "Open a wait dialog""#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(
                requests[0],
                Ok(Request::GuiDialogue {
                    kind: Dialog::Notification { await_close: true },
                    message: String::from("Open a wait dialog")
                })
            )
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_flush() {
    let script = r#"FLUSH"#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0], Ok(Request::TCUFlush))
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_protocol() {
    let script = r#"PROTOCOL"#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0], Ok(Request::None))
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_print() {
    let script = r#"PRINT "t" 123 $F3"#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Ok(Request::TCUTransact(_))));

            if let Ok(Request::TCUTransact(trans)) = request {
                let tx = trans.bytes().to_owned();
                assert_eq!(tx, "P06747BF3\r".as_bytes().to_owned());

                let result = trans.evaluate(&tx);
                assert!(matches!(result, Ok(Request::None)))
            }
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_settimeformat() {
    let script = r#"SETTIMEFORMAT 5"#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Ok(Request::TCUTransact(_))));

            if let Ok(Request::TCUTransact(trans)) = request {
                let tx = trans.bytes().to_owned();
                assert_eq!(tx, "P051B00746605\r".as_bytes().to_owned());

                let result = trans.evaluate(&tx);
                assert!(matches!(result, Ok(Request::None)))
            }
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_setoption() {
    let script = r#"SETOPTION 6 8"#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Ok(Request::TCUTransact(_))));

            if let Ok(Request::TCUTransact(trans)) = request {
                let tx = trans.bytes().to_owned();
                assert_eq!(tx, "P061B00004F0608\r".as_bytes().to_owned());

                let result = trans.evaluate(&tx);
                assert!(matches!(result, Ok(Request::None)))
            }
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_tcuclose() {
    let script = r#"TCUCLOSE 6"#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Ok(Request::TCUTransact(_))));

            if let Ok(Request::TCUTransact(trans)) = request {
                let tx = trans.bytes().to_owned();
                assert_eq!(tx, "C06\r".as_bytes().to_owned());

                let result = trans.evaluate(&tx);
                assert!(matches!(result, Ok(Request::None)))
            }
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_tcuopen() {
    let script = r#"TCUOPEN 2"#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Ok(Request::TCUTransact(_))));

            if let Ok(Request::TCUTransact(trans)) = request {
                let tx = trans.bytes().to_owned();
                assert_eq!(tx, "O02\r".as_bytes().to_owned());

                let result = trans.evaluate(&tx);
                assert!(matches!(result, Ok(Request::None)))
            }
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_tcutest() {
    let script = r#"TCUTEST 3 1000 12000 1 "FAIL""#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Ok(Request::TCUTransact(_))));

            if let Ok(Request::TCUTransact(trans)) = request {
                let tx = trans.bytes().to_owned();
                assert_eq!(tx, "M03\r".as_bytes().to_owned());

                let mut resp = tx;
                let result = trans.evaluate(&resp);
                assert!(matches!(result, Ok(Request::TCUAwaitResponse(_))));

                if let Request::TCUAwaitResponse(trans) = result.unwrap() {
                    resp.extend_from_slice("AA1\r".as_bytes());

                    let result = trans.evaluate(&resp);
                    assert!(matches!(result, Ok(Request::None)))
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

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Ok(Request::TCUTransact(_))));

            if let Ok(Request::TCUTransact(trans)) = request {
                let tx = trans.bytes().to_owned();
                assert_eq!(tx, "P051B00005302\r".as_bytes().to_owned());

                let result = trans.evaluate(&tx);
                assert!(matches!(result, Ok(Request::None)))
            }
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_printertest() {
    let script = r#"PRINTERTEST 3 1000 12000 1 "FAIL""#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Ok(Request::TCUTransact(_))));

            if let Ok(Request::TCUTransact(trans)) = request {
                let tx = trans.bytes().to_owned();
                assert_eq!(tx, "W051B00004D03\r".as_bytes().to_owned());

                let mut resp = tx;
                let result = trans.evaluate(&resp);
                assert!(matches!(result, Ok(Request::TCUAwaitResponse(_))));

                if let Request::TCUAwaitResponse(trans) = result.unwrap() {
                    resp.extend_from_slice("AA1\r".as_bytes());

                    let result = trans.evaluate(&resp);
                    assert!(matches!(result, Ok(Request::None)))
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

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0], Ok(Request::PrinterOpen))
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_usbclose() {
    let script = r#"USBCLOSE"#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0], Ok(Request::PrinterClose))
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_usbprint() {
    let script = r#"USBPRINT "test" 45 $D4"#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            let mut expected = "test".as_bytes().to_owned();
            expected.extend_from_slice(&[45, 0xD4]);

            assert_eq!(requests[0], Ok(Request::PrinterTransmit(expected)))
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_usbsettimeformat() {
    let script = r#"USBSETTIMEFORMAT 6"#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(
                requests[0],
                Ok(Request::PrinterTransmit(vec![0x1B, 0x00, b't', b'f', 6]))
            )
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_usbsetoption() {
    let script = r#"USBSETOPTION 6 7"#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(
                requests[0],
                Ok(Request::PrinterTransmit(vec![0x1B, 0x00, 0x00, b'O', 6, 7]))
            )
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_usbprinterset() {
    let script = r#"USBPRINTERSET 2"#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            assert_eq!(
                requests[0],
                Ok(Request::PrinterTransmit(vec![0x1B, 0x00, 0x00, b'S', 2]))
            )
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_usbprintertest() {
    let script = r#"USBPRINTERTEST 3 1000 12000 1 "FAIL""#;

    match galavant::parse_from_str(script) {
        Ok(exprs) => {
            let requests: Vec<Result<FrontendRequest, Error>> =
                exprs.into_iter().map(galavant::evaluate).collect();

            assert_eq!(requests.len(), 1);
            let request = requests.first().unwrap().to_owned();

            assert!(matches!(request, Ok(Request::PrinterTransact(_))));

            if let Ok(Request::PrinterTransact(trans)) = request {
                let tx = trans.bytes().to_owned();
                assert_eq!(tx, vec![0x1B, 0x00, 0x00, b'M', 3]);

                let resp = "AA1\r".as_bytes();

                let result = trans.evaluate(resp);
                assert!(matches!(result, Ok(Request::None)))
            }
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////
