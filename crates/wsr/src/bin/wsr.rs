use std::process::ExitCode;

fn main() -> ExitCode {
    wsr::main(std::env::args_os())
}
