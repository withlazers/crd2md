use std::{fs::File, io::Write, path::PathBuf};

use clap::Parser;
use crd2md::ToMarkdown;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use serde::Deserialize;
use serde_yaml::Deserializer;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser, Debug)]
struct Opts {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    #[clap(short, long)]
    dir: bool,
}

static ALLOWED_EXTENSIONS: &[&str] = &["yaml", "yml"];

fn main() -> Result<()> {
    let opts = Opts::parse();
    let input = opts.input.unwrap_or(PathBuf::from("-"));
    let input: Vec<Deserializer> = if input == PathBuf::from("-") {
        vec![Deserializer::from_reader(std::io::stdin())]
    } else if input.is_dir() {
        std::fs::read_dir(input)?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                ALLOWED_EXTENSIONS
                    .contains(&path.extension()?.to_str()?)
                    .then_some(path)
            })
            .map(|input| {
                Ok(Deserializer::from_reader(std::fs::File::open(input)?))
            })
            .collect::<Result<Vec<Deserializer>>>()?
    } else {
        vec![Deserializer::from_reader(File::open(input)?)]
    };

    if opts.dir {
        let writer_dir = if let Some(output) = opts.output {
            output
        } else {
            PathBuf::from(".")
        };
        for doc in input.into_iter().flatten() {
            let crd = CustomResourceDefinition::deserialize(doc)?;
            let path = format!("{}.md", crd.spec.names.kind.to_lowercase());
            let mut writer = std::fs::File::create(writer_dir.join(path))?;
            writeln!(writer, "{}", crd.to_markdown())?;
        }
    } else {
        let mut writer: Box<dyn Write> =
            if opts.output == Some(PathBuf::from("-")) {
                Box::new(std::io::stdout())
            } else if let Some(output) = opts.output {
                Box::new(std::fs::File::open(output).unwrap())
            } else {
                Box::new(std::io::stdout())
            };

        let mut first = true;
        for doc in input.into_iter().flatten() {
            if !first {
                writeln!(writer, "\n---\n")?;
            }
            first = false;
            let markdown = CustomResourceDefinition::deserialize(doc)?;
            writeln!(writer, "{}", markdown.to_markdown())?;
        }
    }

    Ok(())
}
