use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use clap::{crate_authors, crate_version, Arg, ArgMatches, Command};
use cpub::{EpubWriter, Metadata};

const arg_id_title: &str = "title";
const arg_id_author: &str = "author";
const arg_id_publisher: &str = "publisher";
const arg_id_published_date: &str = "published-date";
const arg_id_language: &str = "language";
const arg_id_description: &str = "description";
const arg_id_source: &str = "source";
const arg_id_copyright: &str = "copyright";
const arg_id_rtl: &str = "rtl";
const arg_id_tags: &str = "tags";

const batch_number_placeholder: &str = "%num%";

fn main() {
    fn arg_from_id<'a>(
        arg_id: &'a str,
        arg_short: Option<char>,
        value_name: &'a str,
        help_text: &'a str,
        required: bool,
        takes_value: bool,
        multiple_valuse: bool,
    ) -> Arg<'a> {
        let mut output = Arg::new(arg_id)
            .long(arg_id)
            .value_name(value_name)
            .help(help_text)
            .required(required)
            .takes_value(takes_value)
            .multiple_values(multiple_valuse);

        if let Some(d) = arg_short {
            output = output.short(d);
        }
        if multiple_valuse {
            output = output.use_value_delimiter(true);
        }
        return output;
    }

    let common_args = vec![
        arg_from_id(
            arg_id_title,
            Some('t'),
            "TITLE",
            "Set the title",
            true,
            true,
            false,
        ),
        arg_from_id(
            arg_id_author,
            Some('a'),
            "AUTHOR",
            "Set the author",
            true,
            true,
            false,
        ),
        arg_from_id(
            arg_id_publisher,
            Some('p'),
            "PUBLISHER",
            "Set the publisher",
            true,
            true,
            false,
        ),
        arg_from_id(
            arg_id_published_date,
            Some('d'),
            "PUBLISHED-DATE",
            "Set the published date",
            false,
            true,
            false,
        ),
        arg_from_id(
            arg_id_language,
            None,
            "LANGUAGE",
            "Set the language",
            false,
            true,
            false,
        ),
        arg_from_id(
            arg_id_description,
            None,
            "DESCRIPTION",
            "Set the description",
            false,
            true,
            false,
        ),
        arg_from_id(
            arg_id_source,
            None,
            "SOURCE",
            "Set the source",
            false,
            true,
            false,
        ),
        arg_from_id(
            arg_id_copyright,
            None,
            "COPYRIGHT",
            "Set the copyright",
            false,
            true,
            false,
        ),
        arg_from_id(
            arg_id_rtl,
            None,
            "AUTHOR",
            "Set the author",
            false,
            false,
            false,
        ),
        arg_from_id(
            arg_id_tags,
            Some('t'),
            "TAGS",
            "Set the tags",
            false,
            true,
            true,
        ),
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

    if let Some((command, matches)) = matches.subcommand() {
        println!("{}", command);
        if let Some(e) = matches.value_of("title") {
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

fn metadata_from_args(matches: &ArgMatches, batch_number: Option<u32>) -> Metadata {
    let mut output = Metadata::default();
    if let Some(d) = matches.value_of(arg_id_title) {
        output.title = d.to_string();
    }

    return output;
}
