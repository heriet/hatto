use clap::{ArgEnum, Args, Parser, Subcommand};

use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Debug, Subcommand)]
pub enum SubCommand {
    #[clap(name = "evaluate", about = "evaluate policy")]
    Evaluate(EvaluateArgs),
}

#[derive(Debug, Clone, PartialEq, ArgEnum)]
pub enum OutputFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, PartialEq, ArgEnum)]
pub enum SourceType {
    Tsv,
    SpdxTag,
    SpdxJson,
    SpdxYaml,
    // spdx rdf/xls is unsupported
    CycloneDxJson,
    CycloneDxXml,
}

#[derive(Debug, Args)]
pub struct EvaluateArgs {
    #[clap(short = 'p', long = "policy", value_parser, value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
    pub policy: Option<PathBuf>,
    #[clap(short = 'c', long = "curation", value_parser, value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
    pub curation: Option<PathBuf>,
    #[clap(short = 't', long = "source-type", value_parser)]
    pub source_type: Option<SourceType>,
    #[clap(short = 'o', long = "output", value_parser, default_value = "human")]
    pub output_format: OutputFormat,
    #[clap(value_parser, value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
    pub source: PathBuf,
}
