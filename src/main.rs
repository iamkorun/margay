use std::process::ExitCode;

fn main() -> ExitCode {
    match margay::run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("margay: {e:#}");
            ExitCode::from(2)
        }
    }
}
