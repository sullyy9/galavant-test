use gallivant::{FrontendRequest, Interpreter};

pub mod mocks;

////////////////////////////////////////////////////////////////

pub fn interpret_script(script: &str) -> Vec<FrontendRequest> {
    Interpreter::try_from_str(script)
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
}

////////////////////////////////////////////////////////////////
