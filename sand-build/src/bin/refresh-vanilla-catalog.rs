use std::{path::Path, process::ExitCode};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let output = args.next();
    let versions: Vec<_> = args.collect();
    let Some(output) = output.filter(|_| !versions.is_empty()) else {
        eprintln!("usage: refresh-vanilla-catalog <output.json> <minecraft-version> ...");
        return ExitCode::FAILURE;
    };
    match sand_build::refresh_vanilla_catalog(&versions, Path::new(&output)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("failed to refresh the vanilla catalog: {error}");
            ExitCode::FAILURE
        }
    }
}
