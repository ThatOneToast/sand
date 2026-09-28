//! Standalone portable-input adapter: bounded local resolution and publication.
mod input;
use anyhow::Result;
use clap::{Args, Subcommand};
use sand::advanced::compiler::Compiler;
use std::path::PathBuf;

#[derive(Args)]
pub struct ProgramArgs {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Print the generated portable JSON Schema
    Schema {
        #[arg(long, default_value = "json", value_parser = ["json"])]
        format: String,
        /// Generate the standalone module-document schema
        #[arg(long)]
        module: bool,
    },
    /// Report protocol, targets, operations and bounded-work limits
    Capabilities {
        #[arg(long, default_value = "json", value_parser = ["json"])]
        format: String,
    },
    /// Validate a portable program without publishing files
    Check(InputArgs),
    /// Compile and publish a complete pack without Cargo or sand.toml
    Compile {
        #[command(flatten)]
        input: InputArgs,
        #[arg(long)]
        output: PathBuf,
        /// Include UTF-8 file bodies in the machine response
        #[arg(long)]
        include_files: bool,
    },
}
#[derive(Args)]
struct InputArgs {
    /// Envelope file, or - for self-contained stdin
    #[arg(long)]
    input: String,
    /// Explicit module root; defaults to the envelope's parent
    #[arg(long)]
    root: Option<PathBuf>,
    #[arg(long, default_value = "json", value_parser = ["json"])]
    format: String,
}

pub fn run(args: ProgramArgs) -> Result<()> {
    let response = match args.command {
        Command::Schema { module, .. } => {
            serde_json::json!({"schema_version":1,"success":true,"schema": if module {Compiler::module_schema()} else {Compiler::schema()}})
        }
        Command::Capabilities { .. } => {
            serde_json::json!({"schema_version":1,"success":true,"capabilities":Compiler::capabilities()})
        }
        Command::Check(input) => execute(input, None, false),
        Command::Compile {
            input,
            output,
            include_files,
        } => execute(input, Some(output), include_files),
    };
    println!("{}", serde_json::to_string(&response)?);
    if response["success"] == false {
        anyhow::bail!("portable program failed; see JSON diagnostics");
    }
    Ok(())
}
fn execute(input: InputArgs, output: Option<PathBuf>, include_files: bool) -> serde_json::Value {
    let result = input::read(&input.input, input.root.as_deref())
        .and_then(|program| Compiler::compile(&program));
    let pack = match result {
        Ok(pack) => pack,
        Err(diagnostics) => {
            return serde_json::json!({"schema_version":1,"success":false,"diagnostics":diagnostics});
        }
    };
    if let Some(destination) = &output
        && let Err(error) = crate::build::publication::publish_pack(destination, &pack.resources)
    {
        return serde_json::json!({"schema_version":1,"success":false,"diagnostics":[input::error("SAND_PROGRAM_PUBLICATION", &destination.display().to_string(), error)]});
    }
    let files: Vec<_> = pack
        .resources
        .iter()
        .map(|(path, bytes)| serde_json::json!({"path":path,"bytes":bytes.len()}))
        .collect();
    let mut response = serde_json::json!({"schema_version":1,"success":true,"target":pack.target,"diagnostics":pack.diagnostics,"files":files,"output":output});
    if include_files {
        response["file_contents"] = serde_json::to_value(
            pack.resources
                .into_iter()
                .map(|(path, bytes)| {
                    (
                        path,
                        String::from_utf8(bytes).expect("compiler emits UTF-8"),
                    )
                })
                .collect::<std::collections::BTreeMap<_, _>>(),
        )
        .expect("file bodies serialize");
    }
    response
}
