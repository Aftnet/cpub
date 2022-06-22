use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use anyhow::Context;
use chrono::{DateTime, Utc};
use clap::{crate_authors, crate_version, Arg, ArgMatches, Command};
use cpub::{EpubWriter, Metadata};

const CMD_ID_BATCH: &str = "batch";

const ARG_ID_TITLE: &str = "title";
const ARG_ID_AUTHOR: &str = "author";
const ARG_ID_PUBLISHER: &str = "publisher";
const ARG_ID_PUBLISHED_DATE: &str = "published-date";
const ARG_ID_LANGUAGE: &str = "language";
const ARG_ID_DESCRIPTION: &str = "description";
const ARG_ID_SOURCE: &str = "source";
const ARG_ID_COPYRIGHT: &str = "copyright";
const ARG_ID_RTL: &str = "rtl";
const ARG_ID_TAGS: &str = "tags";
const ARG_ID_INPUT: &str = "input";

const VOLUME_NUMBER_PLACEHOLDER: &str = "%num%";

struct App<'a> {
    args: &'a ArgMatches,
}

impl<'a> App<'a> {
    pub fn new(args: &'a ArgMatches) -> Self {
        App::<'a> { args }
    }

    pub fn generate_single(&mut self) {}

    pub fn generate_batch(&mut self) {}

    fn set_metadata_from_args(&self, target: &mut Metadata) -> anyhow::Result<()> {
        if let Some(d) = self.args.value_of(ARG_ID_AUTHOR) {
            target.author = d.to_string();
        }
        if let Some(d) = self.args.value_of(ARG_ID_PUBLISHER) {
            target.publisher = d.to_string();
        }
        if let Some(d) = self.args.value_of(ARG_ID_PUBLISHED_DATE) {
            target.published_date = DateTime::parse_from_rfc3339(d)
                .context("Unable to parse date string")?
                .with_timezone(&Utc);
        }
        if let Some(d) = self.args.value_of(ARG_ID_LANGUAGE) {
            target.language = d.to_string();
        }
        if let Some(d) = self.args.value_of(ARG_ID_DESCRIPTION) {
            target.description = Some(d.to_string());
        }
        if let Some(d) = self.args.value_of(ARG_ID_SOURCE) {
            target.source = Some(d.to_string());
        }
        if let Some(d) = self.args.value_of(ARG_ID_COPYRIGHT) {
            target.copyright = Some(d.to_string());
        }
        if let Some(d) = self.args.values_of(ARG_ID_TAGS) {
            for i in d {
                target.tags.insert(i.to_string());
            }
        }

        target.right_to_left = self.args.is_present(ARG_ID_RTL);
        return Ok(());
    }
}

fn main() {
    fn arg_from_id<'a>(
        arg_id: &'a str,
        arg_short: Option<char>,
        value_name: &'a str,
        help_text: &'a str,
        required: bool,
        positional: bool,
        takes_value: bool,
        multiple_values: bool,
    ) -> Arg<'a> {
        let mut output = Arg::new(arg_id)
            .value_name(value_name)
            .help(help_text)
            .required(required)
            .takes_value(takes_value)
            .multiple_values(multiple_values);

        match positional {
            true => {}
            false => {
                output = output.long(arg_id);
                if let Some(d) = arg_short {
                    output = output.short(d);
                }
            }
        }

        if multiple_values {
            output = output.use_value_delimiter(true);
        }
        return output;
    }

    let common_args = vec![
        arg_from_id(
            ARG_ID_TITLE,
            Some('t'),
            "TITLE",
            "Set the title",
            true,
            false,
            true,
            false,
        ),
        arg_from_id(
            ARG_ID_AUTHOR,
            Some('a'),
            "AUTHOR",
            "Set the author",
            true,
            false,
            true,
            false,
        ),
        arg_from_id(
            ARG_ID_PUBLISHER,
            Some('p'),
            "PUBLISHER",
            "Set the publisher",
            true,
            false,
            true,
            false,
        ),
        arg_from_id(
            ARG_ID_PUBLISHED_DATE,
            Some('d'),
            "PUBLISHED-DATE",
            "Set the published date",
            false,
            false,
            true,
            false,
        ),
        arg_from_id(
            ARG_ID_LANGUAGE,
            None,
            "LANGUAGE",
            "Set the language",
            false,
            false,
            true,
            false,
        ),
        arg_from_id(
            ARG_ID_DESCRIPTION,
            None,
            "DESCRIPTION",
            "Set the description",
            false,
            false,
            true,
            false,
        ),
        arg_from_id(
            ARG_ID_SOURCE,
            None,
            "SOURCE",
            "Set the source",
            false,
            false,
            true,
            false,
        ),
        arg_from_id(
            ARG_ID_COPYRIGHT,
            None,
            "COPYRIGHT",
            "Set the copyright",
            false,
            false,
            true,
            false,
        ),
        arg_from_id(
            ARG_ID_RTL,
            None,
            "RTL",
            "Set the reading order as right to left (manga)",
            false,
            false,
            false,
            false,
        ),
        arg_from_id(
            ARG_ID_TAGS,
            None,
            "TAGS",
            "Set the tags",
            false,
            false,
            true,
            true,
        ),
        arg_from_id(
            ARG_ID_INPUT,
            None,
            "INPUT",
            "Set the input folder",
            true,
            true,
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
            Command::new(CMD_ID_BATCH)
                .version(crate_version!())
                .author(crate_authors!())
                .about(
                    "Create multiple ePubs from directory containig other directories with images",
                )
                .args(&common_args),
        )
        .get_matches();

    match matches.subcommand() {
        Some((CMD_ID_BATCH, matches)) => {
            let mut app = App::new(matches);
            app.generate_batch();
        }
        Some(_) => panic!("Unrecognized parsed command. This should not happen"),
        None => {
            let mut app = App::new(&matches);
            app.generate_single();
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
