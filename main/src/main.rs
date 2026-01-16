use clap::Arg;
use clap::{ArgAction, Command};
use meta::Context;
use std::fs;
use std::path::Path;
use std::process::exit;
use utils::SquashOptions;
use utils::error::{Result, SquashError};

#[tokio::main(worker_threads = 4)]
async fn main() {
    let matches = Command::new("Catsquash")
        .author("Mona, mona@mona.cat")
        .version("1.0.0")
        .subcommand_negates_reqs(true)
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("gzip")
                .short('n')
                .long("no-gzip")
                .action(ArgAction::SetFalse),
        )
        .arg(
            Arg::new("oxipng")
                .short('o')
                .long("oxipng")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("archive_name")
                .required(true)
                .help("The name of the output file, including the file extension.")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("input_dir")
                .required(true)
                .help("The path to the resource pack that should be squashed.")
                .action(ArgAction::Set),
        )
        .get_matches();

    match handle(matches).await {
        Err(err) => {
            eprintln!("{}", err);
            exit(err.into())
        }
        Ok(_) => exit(0),
    }
}

async fn handle(matches: clap::ArgMatches) -> Result<()> {
    let archive_name = Path::new(
        matches
            .get_one::<String>("archive_name")
            .expect("Expected archive name to be present!"),
    );

    let input = Path::new(
        matches
            .get_one::<String>("input_dir")
            .expect("Expected input name to be present!"),
    );

    let options = SquashOptions {
        verbose: matches.get_flag("verbose"),
        gzip: matches.get_flag("gzip"),
        oxipng: matches.get_flag("oxipng"),
    };

    let temp = std::env::temp_dir().join("catsquash");

    if temp.is_dir() && temp.exists() {
        fs::remove_dir_all(&temp).map_err(|err| SquashError::FileError {
            error: err.to_string(),
        })?;
    }

    processors::process(&input, &temp, options.clone()).await?;
    packing::packing::pack(
        &temp,
        archive_name,
        &Context {
            verbose: options.verbose,
            gzip: options.gzip,
        },
    )
    .map_err(|err| SquashError::PackingError(err))?;

    Ok(())
}
