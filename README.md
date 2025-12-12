# gxter

`gxter` is a (WIP) tool for reading and writing GXT files for Rockstar
Games' Grand Theft Auto games, specifically aiming to support GTA III, Vice City
and San Andreas.

In addition, it is also my first experiment for coding in Rust.

**gxter is a work in progress, and support for other useful options is to be added in the future.**

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
  according to the entries in TKEY.

- `-O`, `--offset-sort`: When decompiling, list strings in the order of their
  data offsets, according to the entries in TKEY. This is useful, as in GTA 3
  and VC files, it is possible to determine which strings were created earlier
  or later in the game's development (it is notable that strings related to the
  PC ports of both games are close to the end), and in GTA SA files, since the
  keys are CRC32 hashes, key-based ordering results in pseudorandom arrangement
  of lines, whereas offset-based ordering shows related lines closer to each
  other.

- `-o`, `--output` (argument: file name): When decompiling, output the resulting
  data into a TOML file specified by the argument's value, instead of on screen.
  When compiling, save the GXT file under the following file name.

The first parameter that doesn't fit these will be interpreted as the input file
name.

## Text File Format

The program currently supports the North American and Western European releases
of GTA 3, Vice City and San Andreas. Strings in files will be read and written
according to the character tables built into the program. Support for custom
tables (to be used with, say, Japanese or Russian translations of the games) is
to be added in the future.

The text format is using TOML as a base and has the following structure:
```
format = "format"

[main_table]
NAME = "A string with a name (GTA III / VC format)."
"#01234567" = "A string with a CRC32 hash (GTA SA format)."
[aux_tables.TABLE]
NAME = "A string from an auxiliary table (GTA VC / SA format)."
```

The "format" parameter is a string specifying the format to be used for the
file, with the following values being allowed:

- `Three`: GTA 3 or VC on Xbox
- `Vice`: GTA VC, LCS, VCS
- `San8`: GTA SA / IV, 8-bit characters
- `San16`: GTA IV, 16-bit characters (read as if they're 8-bit)

The `main_table` section lists all the strings in the main table of the GXT
file. In GTA 3 format files, this is the only table. 

The `aux_tables` section lists all the auxiliary tables. These only exist in GTA
VC and SA format files. Their names are fixed-size arrays of 8 bytes, typically
as C-style ASCII strings (null-terminated, if shorter than 8 characters). A
table's name is specified after the period in each `aux_tables.` section. When
imported, these are converted into Rust strings, with extra care to make sure
even abnormal values (like non-zero bytes after zero bytes) are preserved.

Each string can be identified using either its name (GTA 3 / VC files), or CRC32
hash (GTA SA files). Like table names, string names are sequences of 8 bytes.
When imported, string-based names are converted into strings, and hash-based
names are converted into a string of a hash symbol (`#`) followed by the
hexadecimal value of the hash.

GXT file's strings consist of fixed-size characters (8 or 16 bits wide), encoded
using a custom encoding. These are converted into UTF-8 when imported. The
conversion is done using the character tables built into the program. The
characters not specified in the tables are handled as follows:

- Since a null-based byte or wide character is used in GXT files as a string
  terminator, a GXT string can never contain one.
- Characters with values between 1 and 31 will be recorded as equivalent ASCII
  control characters.
- Characters with codes between 32 and 255 (0x00FF) will be recorded as
  Private Use Area characters with corresponding codes between U+E020 and U+E0FF
  (0xE000 will be added to the codepoint).
- Characters with codes between 256 (0x0100) and 65535 (0xFFFF) will be recorded
  as Supplementary Private Use Area-A characters with codes between U+F0000 and
  U+FFEFF (0xFEF00 will be added to the codepoint).

Private Use Area codes are used in order to not imply that an unknown character
matches any existing Unicode character.

## Character Table Format

A character table consists of two tables, a decode table and an encode table.
Normally, just one table is enough, as the encode table can be generated from
the decode table, if omitted. The TOML-based format is as follows:

```
[decode_table]
123='@'
124='#'
[encode_table]
'@'=123
'#'=124
```
In this case, 123 and 124 are the decimal codes for the characters as used in
the GXT file, and the characters to the right are Unicode characters.

If a character is missing from the decode table, it will be decoded according to
the default table for the corresponding format. This means that it is not
necessary to define any unchanged characters in the table file.
