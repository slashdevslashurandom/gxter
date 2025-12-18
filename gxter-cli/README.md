# gxter-cli

`gxter` is a library for reading and writing GXT files for Rockstar
Games' Grand Theft Auto games, specifically aiming to support GTA III, Vice City
and San Andreas.

`gxter-cli` is a command-line utility that uses `gxter` to read and write text
and GXT files.

**gxter-cli is a work in progress, and support for other useful options is to be added in the future.**

The program has been proven capable of decompiling the English language text
files from all three games supported, and compiling them back with small
modifications. Further testing may be necessary to prove whether or not larger
changes will work.

## Usage

The program runs on the command line and works with either GXT files or
TOML-based text files. Use the following arguments to tell what the program
should do:

- `-c`, `--character-table`: Use a custom "character table" in order to convert
  between the game's internal encoding and UTF-8. This option is useful for
  non-standard releases of the games.

- `-d`, `--decompile`: Read a GXT file, then output its contents on the screen
  or into a file specified by the `-o` parameter below.
  If this parameter is not specified, the program will assume the default
  operation is to compile a text file into a GXT file instead. The program will
  determine the GXT's format based on the file's structure and act accordingly.

- `-K`, `--key-sort`: When decompiling, list strings in the order of their keys,
  according to the entries in TKEY. (GXT files are expected by the games to have
  their strings sorted either by key or hash in the TKEY table, so this will
  result in an alphabetical or hash-based sort.)

- `-n`, `--name-list`: When decompiling a GXT file, read a "name list"
  consisting of raw string names. These string names have their CRC32 hashes
  precalculated, and in case one of these is seen in a GTA SA format GXT file,
  that hash is replaced with the name that matches it.

- `-O`, `--offset-sort`: When decompiling, list strings in the order of their
  data *offsets* relative to TDAT, according to the entries in TKEY. This is
  useful, as in GTA 3 and VC files, it is possible to determine which strings
  were originally added earlier or later in the game's development (it is
  notable that strings related to the PC ports of both games are close to the
  end), and in GTA SA files, since the keys are CRC32 hashes, key-based ordering
  results in pseudorandom arrangement of lines, whereas offset-based ordering
  shows related lines closer to each other.

- `-o`, `--output` (argument: file name): When decompiling, output the resulting
  data into a TOML file specified by the argument's value, instead of on screen.
  When compiling, save the GXT file under the following file name.

- `-p`, `--pretty-print`: Instead of converting a text or GXT file, "pretty
  print" its contents in a format designed for terminal output. Color tags in
  the file's strings (e.g. `~r~` for red) will be used to change the text's
  color, and (depending on the format) tags referring to PS2 or Xbox controller
  buttons will be replaced with descriptive labels. **WARNING: since tag
  recognition happens after the strings are decoded into Unicode, this function
  may break if custom character tables are used.**

The first parameter that doesn't fit these will be interpreted as the input file
name.

**See the [README.md file of the original `gxter`
crate](https://github.com/slashdevslashurandom/gxter/README.md) for information on the
file formats used in the application.**

