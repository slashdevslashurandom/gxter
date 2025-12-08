# gxter

`gxter` is a (WIP) tool for decompiling GXT files for Rockstar
Games' Grand Theft Auto games, specifically aiming to support GTA III, Vice City
and San Andreas.

In addition, it is also my first experiment for coding in Rust.

**gxter is a work in progress, and support for compilation of GXT files from
text files is to be added in the future.**

## Usage

The program runs on the command line and works with either GXT files or flat
text files. Use the following arguments to tell what the program should do:

- `-d`, `--decompile`: Read a GXT file, then output its contents on the screen.
  If this parameter is not specified, the program will assume the default
  operation is to compile a text file into a GXT file instead. The program will
  determine the format based on the file's structure and act accordingly.

- `-K`, `--key-sort`: When decompiling, list strings in the order of their keys,
  according to the entries in TKEY.

- `-O`, `--offset-sort`: When decompiling, list strings in the order of their
  data offsets, according to the entries in TKEY.

## Text File Format

The program currently supports the North American and Western European releases
of GTA 3, Vice City and San Andreas. Strings in files will be read according to
the character tables built into the program. Support for custom tables (to be
used with, say, Japanese or Russian translations of the games) is to be added in
the future.

Strings are stored in the following format:
```
#FMT: FORMAT
NAME=Main Table Value
[TABLE]
NAME=Auxiliary Table Value
NAME2=Auxiliary Table Value 2
```

The first line specifies the format to be used for the file, with the word
`FORMAT` being replaced with one of the following values:

- `III`: GTA 3 or VC on Xbox
- `VC`: GTA VC, LCS, VCS
- `SA8`: GTA SA / IV, 8-bit characters
- `SA16`: GTA IV, 16-bit characters (read as if they're 8-bit)

`TABLE` is the name of the strings table in the GXT file. It is only used in VC
and SA format files, as the GTA III format stores every string in a single
table. The table's name is an 8-byte sequence, in which any non-ASCII
characters or open/close brackets will be escaped using the `\xNN` escape
sequence (`NN` being two hexadecimal digits representing the byte's value).

`NAME` is the name of an individual string in a table. For GTA III and VC format
files, it is a sequence of up to 8 bytes, following the same escaping rules as
the table name. For GTA SA format files, it can also consist of a hash sign (`#`)
followed by 8 hexadecimal digits, representing the CRC32 check sum of the actual
string's name. (GTA SA format files only store the checksums of string names,
and these will be saved during decompilation.)

The values are written as UTF-8 strings, converted from the original layout
using the character tables built into the program. Characters that can't be
recognized in the character table are escaped using the `\xNN` (8-bit) or
`\uNNNN` (16-bit) escape sequence, where each letter N is a hexadecimal digit
representing the character value. It is worth noting that, despite the escape
string starting with `\u`, the character represented is not necessarily a
Unicode character.

