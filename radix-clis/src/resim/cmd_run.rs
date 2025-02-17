use crate::resim::*;
use clap::Parser;
use radix_engine::utils::validate_call_arguments_to_native_components;
use radix_transactions::manifest::{
    compile, compiler::compile_error_diagnostics, compiler::CompileErrorDiagnosticsStyle,
    BlobProvider,
};
use regex::{Captures, Regex};
use std::env;
use std::path::PathBuf;

/// Compiles, signs and runs a transaction manifest
#[derive(Parser, Debug)]
pub struct Run {
    /// The path to a transaction manifest file
    pub path: PathBuf,

    /// The network to use when outputting manifest, [simulator | adapanet | nebunet | mainnet]
    #[clap(short, long)]
    pub network: Option<String>,

    /// The paths to blobs
    #[clap(short, long, multiple = true)]
    pub blobs: Option<Vec<String>>,

    /// The private keys used for signing, separated by comma
    #[clap(short, long)]
    pub signing_keys: Option<String>,

    /// Turn on tracing
    #[clap(short, long)]
    pub trace: bool,
}

impl Run {
    pub fn pre_process_manifest(manifest: &str) -> String {
        let re = Regex::new(r"\$\{(.+?)\}").unwrap();
        re.replace_all(manifest, |caps: &Captures| {
            env::var(&caps[1].trim()).unwrap_or_default()
        })
        .into()
    }

    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), String> {
        let manifest = std::fs::read_to_string(&self.path).map_err(Error::IOError)?;
        let pre_processed_manifest = Self::pre_process_manifest(&manifest);
        let network = match &self.network {
            Some(n) => NetworkDefinition::from_str(&n).map_err(Error::ParseNetworkError)?,
            None => NetworkDefinition::simulator(),
        };
        let mut blobs = Vec::new();
        if let Some(paths) = &self.blobs {
            for path in paths {
                blobs.push(std::fs::read(path).map_err(Error::IOError)?);
            }
        }
        let compiled_manifest = compile(
            &pre_processed_manifest,
            &network,
            BlobProvider::new_with_blobs(blobs),
        )
        .map_err(|err| {
            compile_error_diagnostics(
                &pre_processed_manifest,
                err,
                CompileErrorDiagnosticsStyle::TextTerminalColors,
            )
        })?;

        validate_call_arguments_to_native_components(&compiled_manifest.instructions)
            .map_err(Error::InstructionSchemaValidationError)?;

        handle_manifest(
            compiled_manifest,
            &self.signing_keys,
            &self.network,
            &None,
            self.trace,
            true,
            out,
        )
        .map(|_| ())
        .map_err(|err| err.into())
    }
}
