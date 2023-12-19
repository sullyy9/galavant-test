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
            assert_eq!(requests[0], Ok(Request::Nothing))
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
            assert_eq!(requests[0], Ok(Request::Nothing))
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
            assert_eq!(
                requests[0],
                Ok(Request::TCUTransmit(vec![
                    b'P', b'0', b'6', b'7', b'4', b'7', b'B', b'F', b'3'
                ]))
            )
        }
        Err(errors) => panic!("{:?}", errors),
    }
}

////////////////////////////////////////////////////////////////
