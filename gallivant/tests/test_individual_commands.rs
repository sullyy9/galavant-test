use std::time::Duration;

use gallivant::{Dialog, FrontendRequest, TransactionStatus};

type Request = FrontendRequest;

mod common;
use common::{interpret_script, mocks::PortMock};

////////////////////////////////////////////////////////////////

#[test]
fn test_hpmode() {
    let script = r#"HPMODE"#;
    assert_eq!(interpret_script(script), [Request::None]);
}

////////////////////////////////////////////////////////////////

#[test]
fn test_comment() {
    let script = r#"COMMENT "This is a comment 1234""#;
    assert_eq!(
        interpret_script(script),
        [Request::GuiPrint(String::from("This is a comment 1234"))]
    );
}

////////////////////////////////////////////////////////////////

#[test]
fn test_wait() {
    let script = r#"WAIT 12345"#;
    assert_eq!(
        interpret_script(script),
        [Request::Wait(Duration::from_millis(12345))]
    );
}

////////////////////////////////////////////////////////////////

#[test]
fn test_opendialog() {
    let script = r#"OPENDIALOG "Open a dialog""#;
    assert_eq!(
        interpret_script(script),
        [Request::GuiDialogue {
            kind: Dialog::Notification,
            message: String::from("Open a dialog")
        }]
    );
}

////////////////////////////////////////////////////////////////

#[test]
fn test_waitdialog() {
    let script = r#"WAITDIALOG "Open a wait dialog""#;
    assert_eq!(
        interpret_script(script),
        [Request::GuiDialogue {
            kind: Dialog::ManualInput,
            message: String::from("Open a wait dialog")
        }]
    );
}

////////////////////////////////////////////////////////////////

#[test]
fn test_flush() {
    let script = r#"FLUSH"#;
    assert_eq!(interpret_script(script), [Request::TCUFlush]);
}

////////////////////////////////////////////////////////////////

#[test]
fn test_protocol() {
    let script = r#"PROTOCOL"#;
    assert_eq!(interpret_script(script), [Request::None]);
}

////////////////////////////////////////////////////////////////

#[test]
fn test_print() {
    let script = r#"PRINT "t", 123, $F3"#;
    let requests = interpret_script(script);
    assert!(matches!(requests[..], [Request::TCUTransact(_)]));

    if let Request::TCUTransact(mut transaction) = requests[0].clone() {
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

////////////////////////////////////////////////////////////////

#[test]
fn test_settimeformat() {
    let script = r#"SETTIMEFORMAT 5"#;
    let requests = interpret_script(script);
    assert!(matches!(requests[..], [Request::TCUTransact(_)]));

    if let Request::TCUTransact(mut transaction) = requests[0].clone() {
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

////////////////////////////////////////////////////////////////

#[test]
fn test_setoption() {
    let script = r#"SETOPTION 6, 8"#;
    let requests = interpret_script(script);
    assert!(matches!(requests[..], [Request::TCUTransact(_)]));

    if let Request::TCUTransact(mut transaction) = requests[0].clone() {
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

////////////////////////////////////////////////////////////////

#[test]
fn test_tcuclose() {
    let script = r#"TCUCLOSE 6"#;
    let requests = interpret_script(script);
    assert!(matches!(requests[..], [Request::TCUTransact(_)]));

    if let Request::TCUTransact(mut transaction) = requests[0].clone() {
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

////////////////////////////////////////////////////////////////

#[test]
fn test_tcuopen() {
    let script = r#"TCUOPEN 2"#;
    let requests = interpret_script(script);
    assert!(matches!(requests[..], [Request::TCUTransact(_)]));

    if let Request::TCUTransact(mut transaction) = requests[0].clone() {
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

////////////////////////////////////////////////////////////////

#[test]
fn test_tcutest() {
    let script = r#"TCUTEST 3, 1000, 12000, 1, "FAIL""#;
    let requests = interpret_script(script);
    assert!(matches!(requests[..], [Request::TCUTransact(_)]));

    if let Request::TCUTransact(mut transaction) = requests[0].clone() {
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

////////////////////////////////////////////////////////////////

#[test]
fn test_printerset() {
    let script = r#"PRINTERSET 2"#;
    let requests = interpret_script(script);
    assert!(matches!(requests[..], [Request::TCUTransact(_)]));

    if let Request::TCUTransact(mut transaction) = requests[0].clone() {
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

////////////////////////////////////////////////////////////////

#[test]
fn test_printertest() {
    let script = r#"PRINTERTEST 3, 1000, 12000, 1, "FAIL""#;
    let requests = interpret_script(script);
    assert!(matches!(requests[..], [Request::TCUTransact(_)]));

    if let Request::TCUTransact(mut transaction) = requests[0].clone() {
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

////////////////////////////////////////////////////////////////

#[test]
fn test_usbopen() {
    let script = r#"USBOPEN"#;
    assert_eq!(interpret_script(script), [Request::PrinterOpen]);
}

////////////////////////////////////////////////////////////////

#[test]
fn test_usbclose() {
    let script = r#"USBCLOSE"#;
    assert_eq!(interpret_script(script), [Request::PrinterClose]);
}

////////////////////////////////////////////////////////////////

#[test]
fn test_usbprint() {
    let script = r#"USBPRINT "test", 45, $D4"#;
    let requests = interpret_script(script);
    assert!(matches!(requests[..], [Request::PrinterTransact(_)]));

    let mut expected = "test".as_bytes().to_owned();
    expected.extend_from_slice(&[45, 0xD4]);

    if let Request::PrinterTransact(transaction) = requests[0].clone() {
        let mut port = PortMock::new();
        assert_eq!(
            transaction.process(&mut port).unwrap(),
            TransactionStatus::Success
        );

        assert_eq!(port.txdata, expected)
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_usbsettimeformat() {
    let script = r#"USBSETTIMEFORMAT 6"#;
    let requests = interpret_script(script);
    assert!(matches!(requests[..], [Request::PrinterTransact(_)]));

    if let Request::PrinterTransact(transaction) = requests[0].clone() {
        let mut port = PortMock::new();
        assert_eq!(
            transaction.process(&mut port).unwrap(),
            TransactionStatus::Success
        );

        assert_eq!(port.txdata, vec![0x1B, b't', b'f', 6])
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_usbsetoption() {
    let script = r#"USBSETOPTION 6, 7"#;
    let requests = interpret_script(script);
    assert!(matches!(requests[..], [Request::PrinterTransact(_)]));

    if let Request::PrinterTransact(transaction) = requests[0].clone() {
        let mut port = PortMock::new();
        assert_eq!(
            transaction.process(&mut port).unwrap(),
            TransactionStatus::Success
        );

        assert_eq!(port.txdata, vec![0x1B, 0x00, b'O', 6, 7])
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_usbprinterset() {
    let script = r#"USBPRINTERSET 2"#;
    let requests = interpret_script(script);
    assert!(matches!(requests[..], [Request::PrinterTransact(_)]));

    if let Request::PrinterTransact(transaction) = requests[0].clone() {
        let mut port = PortMock::new();
        assert_eq!(
            transaction.process(&mut port).unwrap(),
            TransactionStatus::Success
        );

        assert_eq!(port.txdata, vec![0x1B, 0x00, b'S', 2])
    }
}

////////////////////////////////////////////////////////////////

#[test]
fn test_usbprintertest() {
    let script = r#"USBPRINTERTEST 3, 1000, 12000, 1, "FAIL""#;
    let requests = interpret_script(script);
    assert!(matches!(requests[..], [Request::PrinterTransact(_)]));

    if let Request::TCUTransact(mut transaction) = requests[0].clone() {
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

////////////////////////////////////////////////////////////////
