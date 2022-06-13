use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use clap::{App, Arg, SubCommand};
use cpub::{EpubWriter, Metadata};

fn main() {
    let matches = App::new("Comic ePub maker")
        .version("1.0")
        .author("Aftnet")
        .about("Create ePubs from images")
        .arg(
            Arg::new("title")
                .short('t')
                .long("title")
                .value_name("TITLE")
                .help("Sets the title")
                .takes_value(true)
                .use_delimiter(false),
        )
        .subcommand(
            App::new("batch")
                .about("controls testing features")
                .version("1.3")
                .author("Someone E. <someone_else@other.com>")
                .arg(
                    Arg::new("debug")
                        .short('d')
                        .help("print debug information verbosely"),
                ),
        )
        .get_matches();

    if let Some(d) = matches.subcommand_matches("batch") {
        
    }
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
}
