use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(target) = std::env::args().nth(1) else {
        eprintln!("usage: codegen-ci-version <stable|latest>");
        return ExitCode::FAILURE;
    };

    let version = match target.as_str() {
        "stable" => sand_version::CI_STABLE_CODEGEN_VERSION,
        "latest" => sand_version::LATEST_KNOWN,
        _ => {
            eprintln!("unknown codegen CI target `{target}`; expected `stable` or `latest`");
            return ExitCode::FAILURE;
        }
    };

    println!("{version}");
    ExitCode::SUCCESS
}
