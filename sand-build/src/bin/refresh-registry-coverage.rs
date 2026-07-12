use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (Some(version), Some(output), None) = (args.next(), args.next(), args.next()) else {
        eprintln!("usage: refresh-registry-coverage <minecraft-version> <output.json>");
        return ExitCode::FAILURE;
    };

    match sand_build::refresh_registry_coverage_fixture(&version, &PathBuf::from(output)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("failed to refresh registry coverage for {version}: {error}");
            ExitCode::FAILURE
        }
    }
}
