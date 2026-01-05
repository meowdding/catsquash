use std::fs;
use clap::Arg;
use clap::{ArgAction, Command};
use meta::Context;
use std::path::{Path};

fn main() {
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

    let context = Context {
        verbose: matches.get_flag("verbose"),
        gzip: matches.get_flag("gzip"),
    };

    let temp = std::env::temp_dir().join("catsquash");

    if temp.is_dir() && temp.exists() {
        println!("{}", temp.display());
        fs::remove_dir_all(&temp).expect("meow");
    }

    processors::process(input, &temp, &context).expect("meow");
    packing::packing::pack(&temp, archive_name, &context).expect("meow");
}
