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

## Interface

The library offers an interface for creating `GXTFile` objects from scratch,
reading them from TOML-based text files or existing GXT files. Once created or
read, there are three important fields:

- `format`: an enum that describes the GXT file's format. When exported as GXT,
  the format's rules and limitations will be used.

- `main_table`: an IndexMap that relates key strings (either string names or
  representations of their CRC32 hashes) to value strings (UTF-8 encoded
  representations of the string values).

- `aux_tables`: an IndexMap of other IndexMaps, containing all of the auxiliary
  tables that a GXT file might have.

When importing from or exporting to a GXT file, a *character table* and a *name
list* may be provided. Character tables are used to handle unconventional
mappings between character codes and Unicode characters that don't match the
North American or EFIGS versions of GTA 3 / VC / SA. Name lists are used in GTA
SA format files to replace CRC32 hashes with readable names.

## GXT File Format (description and limitations)

GXT is a binary file format used to store text strings in Grand Theft Auto games
between GTA 2 and IV. The format changes from game to game, and this program
specifically aims to support GTA III, Vice City and San Andreas. The file is
used by the games to derive a list of strings that the games would show on
screen. To switch between supported languages, the games load data from
different files.

The GTA III format file consists of a `TKEY` structure, listing each "key"
(string name) in the file and providing a location to its contents, and a `TDAT`
structure, which provides said contents. The fields storing a string's name are
8 bytes long, but in practice, none of the names are longer than 7 bytes.
(Whether or not it is possible to create a string with an 8-byte name is to be
determined.) A string's text is a sequence of 16-bit "wide characters",
terminated with a double null-byte. (This format is also used by GTA: Vice
City's Xbox port.)

GTA: Vice City's format extends on it by providing a `TABL` structure and
separating the lists of strings into separate tables. Each table is referred to
with a name, which, like names of strings, can theoretically hold up to 8 bytes,
but only 7 are used. Each table now has its own `TKEY` and `TDAT` structures.
(This format is also used by GTA: Liberty City Stories and Vice City Stories.)

GTA: San Andreas modifies the format further. Now it allows for storing either
8- or 16-bit characters (EFIGS versions of SA use 8-bit characters), and names
of strings (but not tables) are replaced with CRC32 (more specifically,
CRC32-JAMCRC) hashes. According to documentation on game scripting (the GTA
Modding wiki in particular), string names can now be up to 39 characters large,
but in practice they still seem to be limited to 7 characters at most. (A
variation on this format is also used by GTA IV.)

Each format uses a different encoding. GTA III and VC's EFIGS versions use an
ASCII-like encoding with select extended characters added afterwards, and some
characters replaced with button or HUD icons. GTA SA's EFIGS release uses a
Windows-1252 encoding. Other releases of the games may use different encodings.

GTA III and VC expect strings in each `TKEY` to be sorted by string name
ASCIIbetically, relying on a binary search to retrieve each string. GTA SA does
the same, but sorts by hash instead of string name. When exporting a new GXT
file, the data offsets will reflect the order of the strings in the text file,
while the `TKEY` entries will always be sorted as the game expects it.

## Text File Format

The program currently supports the North American and Western European releases
of GTA 3, Vice City and San Andreas. Strings in files will be read and written
according to the character tables built into the program. Custom tables may be
used to add support for non-EFIGS versions of the game (for example, the
Japanese release, or any of the bootleg translations) -- in fact, two tables are
provided for common Russian releases of GTA VC and SA.

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
file. In GTA 3 format files, this is the only table, whereas in VC and SA format
files, it's the first table among many.

The `aux_tables` section lists all the auxiliary tables, if they're provided.
Each table's name is specified after the period in each `aux_tables.` section.
When imported, these are converted into Rust strings, with extra care to make
sure even abnormal values (like non-zero bytes after zero bytes) are preserved.

Each string can be identified using either its name or CRC32 hash. When
imported, string-based names are converted into strings, and hash-based names
are converted into a string of a hash symbol (`#`) followed by the hexadecimal
value of the hash. If a string-based name happens to start with a hash symbol,
another one is added to the beginning, in order to prevent name collisions (so,
a string `#01234567` represents a hash of `0x01234567`, and `##01234567`
represents a real string with a hypothetical name `#01234567`, which in turn
would be hashed as `0xEA28B0AB`).

(Note: when a GXT file is being compiled, the reverse operation only happens if
the string for the key contains two hash signs (`##`). Key strings that start
with a single hash sign and aren't followed by exactly 8 other characters will
be read literally, so both `#NAME` and `##NAME` will work to encode a
literal name of `#NAME`)

GXT file's strings consist of fixed-size characters (8 or 16 bits wide), encoded
using a custom encoding. These are converted into UTF-8 when imported. The
conversion is done first using any custom table that may be provided, then using
the character tables built into the program. The characters not specified in the
tables are handled as follows:

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
the GXT file, and the characters inside single quotes are Unicode characters.

If a character is not listed in the decode or encode table, it will be decoded
according to the default table for the corresponding format. This means that it
is not necessary to define any unchanged characters in the table file. This
means that characters that "pull double duty" as both original and modified
characters (for example, Latin `K` and Cyrillic `К`) will be properly encoded
even if only the Cyrillic `К` is defined in the decode or encode table.

## Name List Format

A name list is a simple TOML file consisting of a single field: an array of
strings named "names".

```
names = ["NAME1",
"NAME2",
"QWERTYU",
"IOPASDF",
"GHJKL"]
```

When a name list is loaded, its CRC32 hash values are precalculated, and used to
match hashes from GTA SA format GXT files to said names. The resulting text file
will then display these names instead of hash values.

## TODO

The following functionality is yet to be implemented or tested:

- Try and make sure that tilde-based tags are always decoded *without* use of
  custom tables, to make sure they don't interfere with pretty-printing.
