use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use clap::{crate_authors, crate_version, Arg, Command};
use cpub::{EpubWriter, Metadata};

fn main() {
    let common_args = vec![
        Arg::new("title")
            .short('t')
            .long("title")
            .value_name("TITLE")
            .help("Sets the title")
            .required(true)
            .takes_value(true)
            .multiple_values(false),
        Arg::new("author")
            .short('a')
            .long("author")
            .value_name("AUTHOR")
            .help("Sets the author")
            .required(true)
            .takes_value(true)
            .multiple_values(false),
        Arg::new("publisher")
            .short('p')
            .long("publisher")
            .value_name("PUBLISHER")
            .help("Sets the publisher")
            .required(true)
            .takes_value(true)
            .multiple_values(false),
        Arg::new("published-date")
            .short('d')
            .long("published-date")
            .value_name("PUBLISHED-DATE")
            .help("Sets the date of publication")
            .takes_value(true)
            .multiple_values(false),
        Arg::new("language")
            .long("language")
            .value_name("LANGUAGE")
            .help("Sets the language")
            .takes_value(true)
            .multiple_values(false),
        Arg::new("description")
            .long("description")
            .value_name("DESCRIPTION")
            .help("Sets the description")
            .takes_value(true)
            .multiple_values(false),
        Arg::new("source")
            .long("source")
            .value_name("SOURCE")
            .help("Sets the source")
            .takes_value(true)
            .multiple_values(false),
        Arg::new("copyright")
            .long("copyright")
            .value_name("COPYRIGHT")
            .help("Sets the copyright")
            .takes_value(true)
            .multiple_values(false),
        Arg::new("rtl")
            .short('r')
            .long("rtl")
            .value_name("RTL")
            .help("Sets the reading order as right to left (manga)")
            .takes_value(false),
        Arg::new("tags")
            .long("tags")
            .value_name("TAGS")
            .help("Sets custom tags")
            .takes_value(true)
            .use_value_delimiter(true),
    ];

    let matches = Command::new("Comic ePub maker")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Create single ePub from images in a directory")
        .args(&common_args)
        .subcommand(
            Command::new("batch")
                .version(crate_version!())
                .author(crate_authors!())
                .about(
                    "Create multiple ePubs from directory containig other directories with images",
                )
                .args(&common_args),
        )
        .get_matches();

    if let Some(d) = matches.subcommand() {
        println!("Batch");
        if let Some(e) = d.1.value_of("title") {
            println!("Title {}", e);
        }
    } else {
        println!("Non batch");
        if let Some(e) = matches.value_of("title") {
            println!("Title {}", e);
        }
    }
    /*
    println!("Opening");

    let f = File::create(Path::new("test.epub")).unwrap();
    let f = BufWriter::new(f);

    let mut metadata = Metadata::default();
    //metadata.right_to_left = true;
    let mut writer = EpubWriter::new(f, metadata).unwrap();

    let mut file = File::open(Path::new("test/img01.png")).unwrap();
    writer.set_cover(&mut file).unwrap();

    for i in 0..3 {
        let mut file = File::open(Path::new("test/img01.png")).unwrap();
        writer
            .add_image(&mut file, Option::Some(format!("Bookmark {}", i)))
            .unwrap();
    }

    let mut file = File::open(Path::new("test/img02.png")).unwrap();
    writer.add_image(&mut file, Option::None).unwrap();

    writer.close().unwrap();
     */
}
