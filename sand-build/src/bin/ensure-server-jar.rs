use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(version) = std::env::args().nth(1) else {
        eprintln!("usage: ensure-server-jar <minecraft-version>");
        return ExitCode::FAILURE;
    };
    match sand_build::ensure_server_jar(&version) {
        Ok(path) => {
            println!("{}", path.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("failed to ensure server jar for {version}: {error}");
            ExitCode::FAILURE
        }
    }
}
