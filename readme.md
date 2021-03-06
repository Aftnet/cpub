# cpub

cpub creates fixed layout ePubs from images in a folder, replicating the behavior of cbz/cbr files with the following improvements.

- Support in the most common eBook reader apps and devices
- Better support for landscape, two page reading modes and spreads (wide aspect images spreading over two adjacent pages)
- Standardized, embedded metadata

The process is fully reversible: images are not touched in any way and can be simply extracted back from the generated ePubs (they are zip files with a different extension)

## Usage

By default, the contents of the input directory (i.e. all gif, jpeg or png images) and subdirectories are added to an epub file named after the title set via command line and created in the output directory.

At a minimum, title, author and publisher as well as input and output need to be specified.

The batch subcommand makes cpub scan the input directory and create one volume for each of its top level subdirectories. A unique title (used in the output file name and epub metadata) will be generated by either:
- Replacing %num%, if present in the specified title, with the volume number
- Appending vol. <number> to the title otherwise

`--vsn` and `--vnd` determinw what volume number to start from and how many digits to use when converting it to string (useful to keep alphabetic order when a series has more than 10 volumes)

Use `cpub -h` or `cpub batch -h` for help on supported parameters.

### Example

```
cpub -t "My book title" -a "Author name" -p "Publisher name" /where/my/images/are where/to/create/epub/
```
