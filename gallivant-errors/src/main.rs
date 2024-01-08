use std::fmt::Write;

use ariadne::{Report, Source};

use gallivant::Interpreter;

////////////////////////////////////////////////////////////////

fn main() {
    let mut output = String::new();
    invalid_command(&mut output);
    invalid_argument_type(&mut output);
    invalid_argument_value(&mut output);

    std::fs::write("./errors.txt", &output).unwrap();
    println!("{output}");
}

////////////////////////////////////////////////////////////////

fn invalid_command(output: &mut String) {
    writeln!(output, "invalid_command").unwrap();

    let script = "CMMENT";
    if let Err(errors) = Interpreter::try_from_str(script) {
        for report in errors.into_iter().map(Report::from) {
            let mut buffer = Vec::new();
            report
                .write_for_stdout(Source::from(script), &mut buffer)
                .unwrap();
            output.push_str(&String::from_utf8(buffer).unwrap());
        }
    }
}

////////////////////////////////////////////////////////////////

fn invalid_argument_type(output: &mut String) {
    writeln!(output, "invalid_argument_type").unwrap();

    let script = r#"TCUCLOSE "arg""#;
    if let Err(errors) = Interpreter::try_from_str(script) {
        for report in errors.into_iter().map(Report::from) {
            let mut buffer = Vec::new();
            report
                .write_for_stdout(Source::from(script), &mut buffer)
                .unwrap();
            output.push_str(&String::from_utf8(buffer).unwrap());
        }
    }
}

////////////////////////////////////////////////////////////////

fn invalid_argument_value(output: &mut String) {
    writeln!(output, "invalid_argument_value").unwrap();

    let script = r#"TCUCLOSE 256"#;
    if let Err(errors) = Interpreter::try_from_str(script) {
        for report in errors.into_iter().map(Report::from) {
            let mut buffer = Vec::new();
            report
                .write_for_stdout(Source::from(script), &mut buffer)
                .unwrap();
            output.push_str(&String::from_utf8(buffer).unwrap());
        }
    }
}

////////////////////////////////////////////////////////////////
