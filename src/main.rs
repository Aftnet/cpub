use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Ok, Result};
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
const ARG_ID_RTL: &str = "right-to-left";
const ARG_ID_TAGS: &str = "tags";
const ARG_ID_BATCH_VOLUME_START_NUMBER: &str = "vsn";
const ARG_ID_BATCH_VOLUME_NUM_DIGITS: &str = "vnd";

const ARG_ID_INPUT: &str = "input";
const ARG_ID_OUTPUT: &str = "output";

const VOLUME_NUMBER_PLACEHOLDER: &str = "%num%";

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

    let main_args = [
        arg_from_id(
            ARG_ID_TITLE,
            Some('t'),
            "TITLE",
            "Set the title. Occurrences of '%num%' will be replaced by the volume number in batch mode",
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
            Some('r'),
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
            "INPUT DIRECTORY",
            "Set the input folder",
            true,
            true,
            true,
            false,
        ),
        arg_from_id(
            ARG_ID_OUTPUT,
            None,
            "OUTPUT DIRECTORY",
            "Set the output folder",
            true,
            true,
            true,
            false,
        ),
    ];

    let batch_args = [
        arg_from_id(
            ARG_ID_BATCH_VOLUME_START_NUMBER,
            None,
            "VOLUME-START-NUMBER",
            "Set the first volume number for the batch",
            false,
            false,
            true,
            false,
        ),
        arg_from_id(
            ARG_ID_BATCH_VOLUME_NUM_DIGITS,
            None,
            "VOLUME-NUM-DIGITS",
            "Set the number of digits used to format the volume number when generating titles",
            false,
            false,
            true,
            false,
        ),
    ];

    let matches = Command::new("Comic ePub maker")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Create single ePub from images in a directory")
        .args(&main_args)
        .subcommand(
            Command::new(CMD_ID_BATCH)
                .version(crate_version!())
                .author(crate_authors!())
                .about(
                    "Create multiple ePubs from directory containig other directories with images",
                )
                .args(&batch_args),
        )
        .get_matches();

    match matches.subcommand() {
        Some((CMD_ID_BATCH, batch_matches)) => {
            generate_batch(&matches, batch_matches).unwrap();
        }
        Some(_) => panic!("Unrecognized parsed command. This should not happen"),
        None => {
            generate_single(&matches).unwrap();
        }
    }
}

pub fn generate_single(args: &ArgMatches) -> Result<()> {
    let (inpath, outpath) = io_directories_from_args(args)?;
    let metadata = metadata_from_args(args)?;
    create_epub_file(&metadata, &inpath, &outpath)?;
    return Ok(());
}

pub fn generate_batch(args: &ArgMatches, batch_args: &ArgMatches) -> Result<()> {
    let (inpath, outpath) = io_directories_from_args(args)?;
    let mut metadata = metadata_from_args(args)?;

    let mut vol_dirs = inpath
        .read_dir()?
        .map(|d| d.unwrap().path())
        .filter(|d| d.is_dir())
        .collect::<Vec<_>>();
    vol_dirs.sort();

    if vol_dirs.is_empty() {
        println!("No directories to create volumes from found. Aborting");
        return Ok(());
    }

    let title_pattern = metadata.title.clone();

    let mut vol_ctr = 1u32;
    if let Some(vsn_str) = batch_args.value_of(ARG_ID_BATCH_VOLUME_START_NUMBER) {
        match atoi::atoi::<u32>(vsn_str.as_bytes()) {
            Some(vsn_u32) => vol_ctr = vsn_u32,
            None => println!(
                "Unable to parse start volume number. Defaulting to {}",
                vol_ctr
            ),
        }
    }

    let mut vol_ctr_num_digits = u32::max(
        2u32,
        (((vol_ctr - 1) as usize + vol_dirs.len()) as f32).log10() as u32,
    );
    if let Some(vnd_str) = batch_args.value_of(ARG_ID_BATCH_VOLUME_NUM_DIGITS) {
        match atoi::atoi::<u32>(vnd_str.as_bytes()) {
            Some(vnd_u32) => vol_ctr_num_digits = vnd_u32,
            None => println!(
                "Unable to parse start number of digits for volume number formatting. Defaulting to {}", vol_ctr_num_digits
            ),
        }
    }
    let vol_ctr_fmt_string = format!("0{vol_ctr_num_digits}");

    for vol_dir in vol_dirs.iter() {
        let formatted_vol_number = num_runtime_fmt::NumFmt::from_str(vol_ctr_fmt_string.as_str())
            .unwrap()
            .fmt(vol_ctr)
            .unwrap();
        vol_ctr += 1;

        if title_pattern.matches(VOLUME_NUMBER_PLACEHOLDER).count() > 0 {
            metadata.title = title_pattern.replace(
                VOLUME_NUMBER_PLACEHOLDER,
                format!("{}", formatted_vol_number).as_str(),
            );
        } else {
            metadata.title = format!("{} vol. {}", title_pattern, formatted_vol_number);
        }

        create_epub_file(&metadata, &vol_dir, &outpath)?;
    }

    return Ok(());
}

fn create_epub_file(
    metadata: &Metadata,
    input_dir_path: &Path,
    output_dir_path: &Path,
) -> Result<()> {
    fn create_epub_inner(
        metadata: &Metadata,
        input_dir_path: &Path,
        output_file_path: &Path,
    ) -> Result<()> {
        let f = File::create(output_file_path)?;
        let f = BufWriter::new(f);
        let mut writer = EpubWriter::new(f, metadata.clone())?;

        let image_paths = list_supported_images(input_dir_path)?;
        println!(" ({} images)", image_paths.len());

        let mut cover_set = false;
        let mut ctr = 0;
        for image_path in image_paths.iter() {
            let mut file = BufReader::new(File::open(image_path)?);
            if cover_set {
                writer.add_image(&mut file, None).with_context(|| {
                    format!("Error adding page {}", image_path.to_str().unwrap())
                })?;
            } else {
                writer.set_cover(&mut file).with_context(|| {
                    format!("Error adding cover {}", image_path.to_str().unwrap())
                })?;
                cover_set = true;
            }

            ctr += 1;
            print!(
                "{:4.1}% complete\r",
                (100 * ctr) as f32 / image_paths.len() as f32
            );
        }

        println!("");
        writer.finalize()?;
        return Ok(());
    }

    let mut output_file_path = PathBuf::from(output_dir_path);
    output_file_path.push(format!("{}.epub", metadata.title));
    print!("Generating {}", output_file_path.to_str().unwrap());

    let temp_path = PathBuf::from(format!("{}.epubgen", output_file_path.to_str().unwrap()));
    match create_epub_inner(&metadata, &input_dir_path, &temp_path) {
        anyhow::Result::Ok(()) => {
            std::fs::rename(&temp_path, &output_file_path)?;
            return Ok(());
        }
        Err(d) => {
            if temp_path.exists() {
                std::fs::remove_file(temp_path)?;
            }
            return Err(d);
        }
    }
}

fn list_supported_images(input_dir_path: &Path) -> Result<Vec<PathBuf>> {
    static SUPPORTED_EXTENSIONS: [&'static str; 4] = [".gif", ".jpeg", ".jpg", ".png"];

    let mut output = Vec::<PathBuf>::new();

    let mut dir_paths = input_dir_path
        .read_dir()?
        .map(|d| d.unwrap().path())
        .collect::<Vec<_>>();
    dir_paths.sort();

    let mut subdir_paths = Vec::<PathBuf>::new();
    for i in dir_paths.into_iter() {
        if i.is_file() {
            if SUPPORTED_EXTENSIONS
                .iter()
                .any(|&e| i.to_str().unwrap().ends_with(e))
            {
                output.push(i);
            }
        } else {
            subdir_paths.push(i);
        }
    }

    for i in subdir_paths.into_iter() {
        output.append(&mut list_supported_images(&i)?);
    }

    return Ok(output);
}

fn io_directories_from_args(args: &ArgMatches) -> Result<(PathBuf, PathBuf)> {
    let inpath = PathBuf::from(args.value_of(ARG_ID_INPUT).unwrap());
    if !(inpath.exists() && inpath.is_dir()) {
        return Err(anyhow!("Input path is not a directory or does not exist",));
    }

    let outpath = PathBuf::from(args.value_of(ARG_ID_OUTPUT).unwrap());
    if !(outpath.exists() && outpath.is_dir()) {
        return Err(anyhow!("Output path is not a directory or does not exist",));
    }

    return Ok((inpath, outpath));
}

fn metadata_from_args(args: &ArgMatches) -> Result<Metadata> {
    let mut output = Metadata::default();

    if let Some(d) = args.value_of(ARG_ID_TITLE) {
        output.title = d.to_string();
    }
    if let Some(d) = args.value_of(ARG_ID_AUTHOR) {
        output.author = d.to_string();
    }
    if let Some(d) = args.value_of(ARG_ID_PUBLISHER) {
        output.publisher = d.to_string();
    }
    if let Some(d) = args.value_of(ARG_ID_PUBLISHED_DATE) {
        output.published_date = DateTime::parse_from_rfc3339(d)
            .context("Unable to parse date string")?
            .with_timezone(&Utc);
    }
    if let Some(d) = args.value_of(ARG_ID_LANGUAGE) {
        output.language = d.to_string();
    }
    if let Some(d) = args.value_of(ARG_ID_DESCRIPTION) {
        output.description = Some(d.to_string());
    }
    if let Some(d) = args.value_of(ARG_ID_SOURCE) {
        output.source = Some(d.to_string());
    }
    if let Some(d) = args.value_of(ARG_ID_COPYRIGHT) {
        output.copyright = Some(d.to_string());
    }
    if let Some(d) = args.values_of(ARG_ID_TAGS) {
        for i in d {
            output.tags.insert(i.to_string());
        }
    }

    output.right_to_left = args.is_present(ARG_ID_RTL);
    output.validate()?;
    return Ok(output);
}
