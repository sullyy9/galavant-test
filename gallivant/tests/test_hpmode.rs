use gallivant::{Error, FrontendRequest, Interpreter};

type Request = FrontendRequest;

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
            let request = requests[1].clone();

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
fn test_hpmode_setoption() {
    let script = r#"
HPMODE
SETOPTION 6, 8
    "#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();

            assert_eq!(requests.len(), 2);
            let request = requests[1].clone();

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
fn test_hpmode_printerset() {
    let script = r#"
HPMODE    
PRINTERSET 2
    "#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();

            assert_eq!(requests.len(), 2);
            let request = requests[1].clone();

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
fn test_hpmode_printertest() {
    let script = r#"
HPMODE
PRINTERTEST 3, 1000, 12000, 1, "FAIL"
"#;

    match Interpreter::try_from_str(script) {
        Ok(interpreter) => {
            let requests: Vec<Result<FrontendRequest, Error>> = interpreter.collect();

            assert_eq!(requests.len(), 2);
            let request = requests[1].clone();

            assert!(matches!(request, Ok(Request::TCUTransact(_))));

            if let Ok(Request::TCUTransact(trans)) = request {
                let tx = trans.bytes().to_owned();
                assert_eq!(tx, "W051B00004D03\r".as_bytes().to_owned());

                let result = trans.evaluate(&tx);
                assert!(matches!(result, Ok(Request::TCUAwaitResponse(_))));

                if let Request::TCUAwaitResponse(trans) = result.unwrap() {
                    let result = trans.evaluate("AA1\r".as_bytes());
                    assert!(matches!(result, Ok(Request::None)))
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
            assert_eq!(
                requests[1],
                Ok(Request::PrinterTransmit(vec![0x1B, 0x00, b't', b'f', 6]))
            )
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
            assert_eq!(
                requests[1],
                Ok(Request::PrinterTransmit(vec![0x1B, 0x00, 0x00, b'O', 6, 7]))
            )
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
            assert_eq!(
                requests[1],
                Ok(Request::PrinterTransmit(vec![0x1B, 0x00, 0x00, b'S', 2]))
            )
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
            let request = requests[1].clone();

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
