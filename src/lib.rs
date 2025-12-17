use crc32_light::crc32;
use std::io::prelude::*;
use thiserror::Error;
use indexmap::IndexMap;
use std::collections::HashMap;

#[derive(serde::Serialize,serde::Deserialize,Clone)]
/// Specifies one of the possible formats to be used when creating or loading a GXT file
pub enum GXTFileFormat {
    /// GTA III, GTA: Vice City (Xbox)
    Three,

    /// GTA: Vice City, Liberty City Stories, Vice City Stories
    Vice,

    /// GTA: San Andreas, GTA IV (8-bit character data)
    San8,

    /// GTA IV (16-bit character data)
    San16,
}

/// Specifies the order in which strings are to be stored, when read from a GXT file
pub enum ImportOrdering {
    /// Do not change the order during import (order according to TDAT and TKEY entries)
    Native, 

    /// Sort tables and strings sorted by their names (alphabetically or by CRC32 hash)    
    Key,    

    /// Sort tables and strings sorted by their data offsets in the GXT file
    Offset, 
}

/// Describes the possible errors that can be returned by the program
#[derive(Error, Debug)]
pub enum GXTError {
    /// Error during parsing of a GXT file
    #[error("GXT file parsing error: {0}")]
    ParsingError(String),
    /// Error during compilation of a GXT file
    #[error("GXT file compilation error: {0}")]
    CompilationError(String),
    /// Error from the I/O functions
    #[error("I/O error")]
    IOError(#[from] std::io::Error),
    /// Error from the TOML serializer
    #[error("TOML serialization error")]
    TOMLSerError(#[from] toml::ser::Error),
    /// Error from the TOML deserializer
    #[error("TOML deserialization error")]
    TOMLDeError(#[from] toml::de::Error),
}

#[derive(serde::Serialize,serde::Deserialize)]
pub struct GXTFile {

    /// Specifies the format used when decompiling or compiling the GXT file.
    pub format: GXTFileFormat,

    /// Contains the "main" table. In GTA 3 files, this is the only table, whereas in GTA VC and SA
    /// files, it is the first table in the file. The key is the string's name (or hexadecimal hash
    /// prefixed by #), the value is the actual string value, mapped to UTF-8.
    pub main_table: IndexMap<String,String>,

    /// Contains all the "auxiliary" tables. This container must be empty when working with GTA 3
    /// files. The default ordering when decompiling a GXT file is to follow the list as
    /// specified in the TABL section. The key is the table's name, the value is an IndexMap of
    /// name/string values, same as in main_table.
    #[serde(default)]
    #[serde(skip_serializing_if = "aux_tables_are_empty")]
    pub aux_tables: IndexMap<String,IndexMap<String,String>>,

    ///// Contains a list of names found in the file that were successfully mapped to CRC32 hashes. 
    //#[serde(skip)]
    //pub name_list: HashMap<u32,String>
}

/// This structure contains a custom character table that can be used to convert between GXT and
/// text formats for non-NA/EFIGS versions of the games.
#[derive(serde::Serialize,serde::Deserialize)]
pub struct GXTCharacterTable {

    /// This is the primary table. It will be used when decoding characters from GXT to figure out,
    /// which of them needs to be written into the TOML file.
    pub decode_table: HashMap<u16, char>,

    /// This is the encode table, used to determine how characters might be encoded. The reason for
    /// the two tables to exist is that due to how some of games' translations are unofficial, they
    /// may reuse the same character code for two different similar-looking characters, like the
    /// digit "3" for the Cyrillic letter "З" or the latin "k" for the Cyrillic "к" -- but when
    /// editing a text file, it is best to allow both to be resolved into the same character when
    /// exporting as GXT. If not specified, the encode table will be built from the decode table.
    #[serde(default)]
    pub encode_table: HashMap<char, u16>,
}

/// helper function used to avoid serializing aux_tables if there are none
fn aux_tables_are_empty(table: &IndexMap<String,IndexMap<String,String>>) -> bool {
    return table.len() == 0;
}

// -- internal structures, not recommended for use

/// Describes how a string's name may be encoded in the GXT file
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
enum GXTStringName {
    /// Text format (III / VC)
    Text([u8;8]),
    /// CRC32 format (SA)
    CRC32(u32),
}

// these tables contain the default US/European character tables used by GTA 3, VC and SA.
// the first 32 elements are skipped in all of them. the tables largely match ASCII, but
// then add extra accented characters for EFIGS support or modify certain characters to add
// icons for PS2 controller buttons or the HUD.
//
// null characters in this array are treated by the decode_character function as an indication
// that the character needs to be escaped using the private use area.

const GTA3_DEFAULT_CHARACTER_TABLE: [char; 224] = [
    ' ', '!', '"', '#', '$', '%', '&','\'', '(', ')', '*', '+', ',', '-', '.', '/',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
    '™', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\',']', '^', '°',
    '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '❤', '◯','\0', '~','\0',
    'À', 'Á', 'Â', 'Ä', 'Æ', 'Ç', 'È', 'É', 'Ê', 'Ë', 'Ì', 'Í', 'Î', 'Ï', 'Ò', 'Ó',
    'Ô', 'Ö', 'Ù', 'Ú', 'Û', 'Ü', 'ß', 'à', 'á', 'â', 'ä', 'æ', 'ç', 'è', 'é', 'ê',
    'ë', 'ì', 'í', 'î', 'ï', 'ò', 'ó', 'ô', 'ö', 'ù', 'ú', 'û', 'ü', 'Ñ', 'ñ', '¿',
    '¡','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0',
   '\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0',
   '\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0',
   '\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0',
   '\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0',
];

const VICE_DEFAULT_CHARACTER_TABLE: [char; 224] = [
    ' ', '!', '"', '#', '$', '%', '&','\'', '(', ')', '*', '+', ',', '-', '.', '/',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '🛡', '=', '★', '?',
    '™', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\',']', '¡', '°',
    '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '❤', '|', '}', '~','\0',
    'À', 'Á', 'Â', 'Ä', 'Æ', 'Ç', 'È', 'É', 'Ê', 'Ë', 'Ì', 'Í', 'Î', 'Ï', 'Ò', 'Ó',
    'Ô', 'Ö', 'Ù', 'Ú', 'Û', 'Ü', 'ß', 'à', 'á', 'â', 'ä', 'æ', 'ç', 'è', 'é', 'ê',
    'ë', 'ì', 'í', 'î', 'ï', 'ò', 'ó', 'ô', 'ö', 'ù', 'ú', 'û', 'ü', 'Ñ', 'ñ', '¿',
   '\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0',
   '\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0',
   '\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0',
   '\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0',
   '\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0','\0',
]; //the additional letter characters are temporarily disabled, until i can figure out how to
   //separate them from the regular ones

const SAN_DEFAULT_CHARACTER_TABLE: [char; 224] = [ //this is just the CP1252 codepage
    ' ', '!', '"', '#', '$', '%', '&','\'', '(', ')', '*', '+', ',', '-', '.', '/',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
    '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\',']', '^', '_',
    '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~','\0',
    '€','\0', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ','\0', 'Ž','\0',
   '\0', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ','\0', 'ž', 'Ÿ',
    ' ', '¡', '¢', '£', '¤', '¥', '¦', '§', '¨', '©', 'ª', '«', '¬', '­', '®', '¯',
    '°', '±', '²', '³', '´', 'µ', '¶', '·', '¸', '¹', 'º', '»', '¼', '½', '¾', '¿',
    'À', 'Á', 'Â', 'Ã', 'Ä', 'Å', 'Æ', 'Ç', 'È', 'É', 'Ê', 'Ë', 'Ì', 'Í', 'Î', 'Ï',
    'Ð', 'Ñ', 'Ò', 'Ó', 'Ô', 'Õ', 'Ö', '×', 'Ø', 'Ù', 'Ú', 'Û', 'Ü', 'Ý', 'Þ', 'ß',
    'à', 'á', 'â', 'ã', 'ä', 'å', 'æ', 'ç', 'è', 'é', 'ê', 'ë', 'ì', 'í', 'î', 'ï',
    'ð', 'ñ', 'ò', 'ó', 'ô', 'õ', 'ö', '÷', 'ø', 'ù', 'ú', 'û', 'ü', 'ý', 'þ', 'ÿ',
];

/// This function returns a bitwise NOT of the CRC32 algorithm, in order to match the CRC32-JAMCRC
/// algorithm that the GTA: San Andreas code is actually performing.
fn crc32_jamcrc(bin_data: &[u8]) -> u32 {
    !crc32(bin_data)
}

fn decode_character(character_value: u16, format: &GXTFileFormat, custom_table: &Option<GXTCharacterTable>) -> char {

    let character_table: [char; 224] = match format {
        GXTFileFormat::Three => GTA3_DEFAULT_CHARACTER_TABLE,
        GXTFileFormat::Vice => VICE_DEFAULT_CHARACTER_TABLE,
        GXTFileFormat::San8 | GXTFileFormat::San16 => SAN_DEFAULT_CHARACTER_TABLE,
    };

    if character_value < 32 {
        char::from_u32(character_value.into()).unwrap()
    } else {

        let default_value = if character_value >= 0x100 {
            char::from_u32(0xFEF00 + (character_value as u32) ).unwrap()
        } else {
            char::from_u32(0xE000 + character_value as u32).unwrap()
        };

        match custom_table {
            Some(v) => {
                let table_value: Option<&char> = v.decode_table.get(&character_value);
                if let Some(i) = table_value {
                    return *i;
                }
            },
            None => {},
        }

        if ((character_value - 32) as usize) < character_table.len() {
            let table_value = character_table[usize::from(character_value) - 32];
            if table_value != '\0' { table_value } else { default_value }
        } else { default_value }
    }
}

fn encode_character(character: char, format: &GXTFileFormat, custom_table: &Option<GXTCharacterTable>) -> Result<u16,GXTError> {
    
    let character_table: [char; 224] = match format {
        GXTFileFormat::Three => GTA3_DEFAULT_CHARACTER_TABLE,
        GXTFileFormat::Vice => VICE_DEFAULT_CHARACTER_TABLE,
        GXTFileFormat::San8 | GXTFileFormat::San16 => SAN_DEFAULT_CHARACTER_TABLE,
    };

    let char_code = character as u32;
    if char_code < 32 { //characters between 0 and 31
        Ok(char_code.try_into().unwrap())
    } else if (char_code >= 0xE020) && (char_code <= 0xE0FF) { //PUA-based code for 32~255
        Ok((char_code - 0xE000).try_into().unwrap()) 
    } else if (char_code >= 0xF0000) && (char_code <= 0xFFEFF) { //PUA-based code for 16-bit chars
        Ok((char_code - 0xFEF00).try_into().unwrap())
    } else {
        
        if let Some(v) = custom_table {
            let table_value: Option<&u16> = v.encode_table.get(&character);
            if let Some(i) = table_value {
                if *i != 0 {return Ok(*i)};
            }
        }

        for item in character_table.into_iter().enumerate() {
            let (i, c) : (usize, char) = item;
            if (c as u32) == char_code { return Ok(32 + (i as u16)); }
        }
        return Err(GXTError::CompilationError(format!("Codepoint with incompatible value U+{:04X} found",u32::from(character))));
    }
}

fn encode_string(string: &str, format: &GXTFileFormat, custom_table: &Option<GXTCharacterTable>) -> Result<Vec<u8>,GXTError> {

    let mut res: Vec<u8> = vec!();

    match format {
        GXTFileFormat::San8 => {
            for e in string.chars() {
                let widechar = encode_character(e, format, custom_table)?;
                if widechar >= 256 {
                    return Err(GXTError::CompilationError(format!("Character U+{:04X} is to be encoded as {:04X}, but the 8-bit format GXT file can only encode characters below 255.",u32::from(e),widechar)));
                }
                res.push((widechar & 0xFF) as u8);
            }
            res.push(0); // null-terminator
        },
        GXTFileFormat::Three | GXTFileFormat::Vice | GXTFileFormat::San16 => {
            for e in string.chars() {
                let widechar: u16 = encode_character(e, format, custom_table)?;
                res.extend_from_slice(&u16::to_le_bytes(widechar));
            }
            res.extend_from_slice(&[0,0]); //null-terminator
        },
    };

    Ok(res)
}

pub fn read_name_list(file: &mut (impl Read + std::io::Seek + std::io::BufRead)) -> Result<HashMap<u32,String>,GXTError> {

    let mut raw_data: String = Default::default();
    file.read_to_string(&mut raw_data)?;

    #[derive(serde::Deserialize)]
    struct NameList {
        names: Vec<String>
    }
        
    let raw_table: NameList = toml::from_str(&raw_data)?;

    let mut table: HashMap<u32,String> = Default::default();

    for e in raw_table.names {
        let _ = &table.insert(crc32_jamcrc(e.as_bytes()), e);
    }

    Ok(table)
}

pub fn read_custom_table(file: &mut (impl Read + std::io::Seek + std::io::BufRead)) -> Result<GXTCharacterTable,GXTError> {

    let mut raw_data: String = Default::default();
    file.read_to_string(&mut raw_data)?;
        
    let mut table: GXTCharacterTable = toml::from_str(&raw_data)?;

    // If there's no encode table, build one using the decode table
    if table.encode_table.len() == 0 {
        for (k,v) in &table.decode_table {
            table.encode_table.entry(*v).or_insert(*k);
        }
    }

    return Ok(table);
}

#[derive(Clone)]
struct GXTInternalTKEY {
    name: Option<[u8;8]>, //None for GTA 3 or MAIN block in VC
    offset: u32, //location of the TKEY entry in the file
    size: u32, //number of bytes, not number of entries!
    entries: Vec<GXTInternalTKEYEntry>,
}

#[derive(Clone)]
struct GXTInternalTKEYEntry { 
    offset: u32, //TDAT entry offset, relative to the first entry in the array
    name: GXTStringName, //name of the text entry
}

#[derive(Clone)]
struct GXTInternalTABLEntry {
    name: [u8;8],
    offset: u32,
    is_main: bool, //whether or not this is the main table, the first one read
}

#[derive(Clone)]
struct GXTInternalTABL {
    size: u32, //number of elements
    entries: Vec<GXTInternalTABLEntry>, //array of names and offsets
}

struct GXTCompilationTDAT {
    // this buffer will store the actual contents of TDAT. it will be gradually filled with new
    // strings
    buffer: Vec<u8>,
    // this hashmap will store offsets to each individual string and keep track of which ones
    // already exist. the keys are string VALUES, not string names
    offset_map: HashMap<String, usize>,
}

/// returns a sanitized string name from a raw 8-byte token name
fn string_from_name(name: &GXTStringName, name_list: &Option<HashMap<u32,String>>) -> String {

    match name {
        GXTStringName::Text(t) => {
            // this complicated code's goal is to catch potential names that have zero bytes followed by
            // nonzero bytes
            let mut last_nonzero_index: Option<usize> = None;
            
            for i in 0..t.len() {
                let c = t[i];
                if (c != 0) && ((last_nonzero_index == None) || (i > last_nonzero_index.unwrap())) { 
                    last_nonzero_index = Some(i); 
                }
            }

            match last_nonzero_index {
                None => {return "".to_string();},
                Some(l) => {
                    let mut ret:String = String::new();
                    for i in 0..=l { //inclusive range!
                        ret.push(t[i] as char);
                    }
                    return ret;
                },
            };
        },
        GXTStringName::CRC32(c) => {
            match name_list {
                Some(l) => match l.get(c) {
                        Some(s) => s.to_string(),
                        None => format!("#{c:08X}"),
                    },
                None => format!("#{c:08X}"),
            }
        }
    }
}

// used for both III / VC string names and table names
fn string_to_name_basic(string: &str) -> Result<[u8;8],GXTError> {
    let mut encoded_string: [u8;8] = [0;8];
    if string.as_bytes().len() > 8 {
        return Err(GXTError::CompilationError(format!("String name ({}) can't be longer than 8 bytes",string)));
    }
    let len = string.as_bytes().len();

    encoded_string[0..len].copy_from_slice(string.as_bytes());
    return Ok(encoded_string);
}

fn string_to_name_crc32(string: &str) -> Result<u32,GXTError> {
    // if the string resembles a CRC32, read the hexadecimal value!
    if (string.chars().nth(0).unwrap() == '#') && (string.len() == 9) {
        if !string.is_ascii() { return Err(GXTError::CompilationError(format!("Invalid characters in hash-based string ({})",string))); }
        let mut hex_hash: [u8; 8] = [0;8];
        hex_hash[0..8].copy_from_slice(&string.as_bytes()[1..9]);
        let mut raw_hash: [u8; 4] = [0;4];
        match hex::decode(hex_hash) {
            Ok(v) => { 
                raw_hash[0..4].copy_from_slice(&v.as_slice()[0..4]);
            },
            Err(_e) => { 
                return Err(GXTError::CompilationError(format!("Hash-based string ({}) does not contain a valid hex value",string)));
            }
        };
        let hash: u32 = u32::from_be_bytes(raw_hash);
        return Ok(hash);
    } else {
        // get a CRC32 hash from an existing string
        return Ok(crc32_jamcrc(string.as_bytes()));
    }
}

fn string_to_name(string: &str, format: &GXTFileFormat) -> Result<GXTStringName,GXTError> {
    match format {
        GXTFileFormat::Three | GXTFileFormat::Vice => { // string names are 8-byte sequences
            return Ok(GXTStringName::Text(string_to_name_basic(string)?));
        },
        GXTFileFormat::San8 | GXTFileFormat::San16 => { // string names are CRC32s
            return Ok(GXTStringName::CRC32(string_to_name_crc32(string)?));
        },
    }
}

fn gxt_read_tabl(file: &mut (impl Read + std::io::Seek)) -> Result<GXTInternalTABL,GXTError> {

    let mut magic_number: [u8; 4] = [0;4];
    file.read_exact(&mut magic_number)?;
    
    if magic_number != *b"TABL" {
        return Err(GXTError::ParsingError("Invalid TABL header".to_string()));
    }

    let mut tabl = GXTInternalTABL {
        size: 0,
        entries: Vec::new(),
    };
    
    let mut raw_size: [u8; 4] = [0;4];
    file.read_exact(&mut raw_size)?;

    tabl.size = u32::from_le_bytes(raw_size);
    let count = u32::from_le_bytes(raw_size) / 12; //each TABL entry is 12 bytes long
    let mut index: u32 = 0;
    
    while index < count {
        let mut raw_name: [u8; 8] = [0;8];
        let mut raw_offset: [u8; 4] = [0;4];
        
        file.read_exact(&mut raw_name)?;
        file.read_exact(&mut raw_offset)?;

        let offset = u32::from_le_bytes(raw_offset);

        tabl.entries.push(GXTInternalTABLEntry { name:raw_name, offset:offset, is_main: (index == 0) && (raw_name == *b"MAIN\0\0\0\0") });

        index += 1;
    }

    return Ok(tabl);

}

fn gxt_read_tkey(file: &mut (impl Read + std::io::Seek), format: &GXTFileFormat, name: Option<[u8;8]>, offset:Option<u32>, ordering: &Option<ImportOrdering>) -> Result<GXTInternalTKEY,GXTError> {
    //name should be None for GTA3 and VC's MAIN entry

    file.seek(std::io::SeekFrom::Start(offset.unwrap_or(0).into()))?;

    let actual_name: Option<[u8;8]> = match name {
        None => None,
        Some(_) => {
            let mut raw_name: [u8;8] = [0;8];
            file.read_exact(&mut raw_name)?;
            Some(raw_name)
        },
    };
    
    let mut magic_number: [u8; 4] = [0;4];
    file.read_exact(&mut magic_number)?;
    
    if magic_number != *b"TKEY" {
        return Err(GXTError::ParsingError("Invalid TKEY header".to_string()));
    }

    let mut tkey = GXTInternalTKEY {
        name: actual_name,
        offset: offset.unwrap_or(0),
        size: 0,
        entries: Vec::new(),
    };

    let mut raw_size: [u8; 4] = [0;4];
    file.read_exact(&mut raw_size)?;

    tkey.size = u32::from_le_bytes(raw_size);
    
    let entry_size = match format {
        GXTFileFormat::Three | GXTFileFormat::Vice => 12, //4 for offset, 8 for name
        GXTFileFormat::San8 | GXTFileFormat::San16 => 8, //4 for offset, 4 for CRC32
    };
    let count = u32::from_le_bytes(raw_size) / entry_size; //each TKEY entry is 12 bytes long
    let mut index: u32 = 0;

    while index < count {
        
        let mut raw_offset: [u8; 4] = [0;4];
        file.read_exact(&mut raw_offset)?;
        let offset = u32::from_le_bytes(raw_offset);
        
        let name: GXTStringName = match format {
            GXTFileFormat::Three | GXTFileFormat::Vice => {
                let mut raw_name: [u8; 8] = [0;8];
                file.read_exact(&mut raw_name)?;
                GXTStringName::Text(raw_name)
            },
            GXTFileFormat::San8 | GXTFileFormat::San16 => {
                let mut raw_crc32: [u8; 4] = [0;4];
                file.read_exact(&mut raw_crc32)?;
                GXTStringName::CRC32(u32::from_le_bytes(raw_crc32))
            },
        };

        tkey.entries.push(GXTInternalTKEYEntry { offset, name });

        index += 1;
    }
    
    match ordering {
        None | Some(ImportOrdering::Native) => {},
        Some(ImportOrdering::Key) => {
            tkey.entries.sort_by(|a,b| a.name.cmp(&b.name));
        },
        Some(ImportOrdering::Offset) => {
            tkey.entries.sort_by(|a,b| a.offset.cmp(&b.offset));
        },
    }


    return Ok(tkey);
}

fn gxt_read_tdat(file: &mut (impl Read + std::io::Seek), tkey: &GXTInternalTKEY, tkey_offset: Option<u32>, format: &GXTFileFormat, ordering: &Option<ImportOrdering>, custom_table: &Option<GXTCharacterTable>, name_list: &Option<HashMap<u32, String>>) -> Result<IndexMap<String,String>,GXTError> {
    
    let mut tkey_data_sorted = tkey.entries.clone();
    tkey_data_sorted.sort_by(|a,b| a.offset.cmp(&b.offset));

    let mut key_ordering:  Vec<String> = Vec::new();
    let mut offset_ordering: Vec<String> = Vec::new();

    let tdat_offset = tkey_offset.unwrap_or(0) + tkey.size + 8 + match tkey.name {
        None => 0, //MAIN block doesn't have the extra 8 bytes at the start
        Some(_) => 8}; //named blocks do

    file.seek(std::io::SeekFrom::Start(tdat_offset.into()))?;

    let mut magic_number: [u8; 4] = [0;4];
    file.read_exact(&mut magic_number)?;
    
    if magic_number != *b"TDAT" {
        return Err(GXTError::ParsingError("Invalid TDAT header".to_string()));
    }

    let mut raw_size: [u8; 4] = [0;4];
    file.read_exact(&mut raw_size)?;

    let mut table = IndexMap::<String,String>::new();
    let mut offset_table = HashMap::<String,u64>::new();

    for e in &tkey.entries {
        let name = string_from_name(&e.name, name_list);
        let offset: u64 = (tdat_offset + 8 + e.offset).into();
        //eprintln!("Entry offset for {name} is {}, seeking to {offset}...", e.offset);
        
        file.seek(std::io::SeekFrom::Start(offset))?;
                
        let mut value = String::new();

        match format {
            GXTFileFormat::Three | GXTFileFormat::Vice => {
                let mut raw_2byte_sequence: [u8; 2] = [0;2];

                loop {
                    file.read_exact(&mut raw_2byte_sequence)?;
                    let character_value = raw_2byte_sequence[0] as u16 + 256*(raw_2byte_sequence[1] as u16);
                    if character_value == 0 { break; }
                    value.push(decode_character(character_value,&format,custom_table));
                };
            },
            GXTFileFormat::San8 => {
                let mut raw_byte: [u8; 1] = [0];
                loop {
                    file.read_exact(&mut raw_byte)?;
                    if raw_byte[0] == 0 { break; }
                    value.push(decode_character(raw_byte[0].into(),&format,custom_table));
                };
            },
            GXTFileFormat::San16 => {
                let mut raw_2byte_sequence: [u8; 2] = [0;2];

                loop {
                    file.read_exact(&mut raw_2byte_sequence)?;
                    let character_value = raw_2byte_sequence[0] as u16;
                    if character_value == 0 { break; }
                    value.push(decode_character(character_value,&format,custom_table));
                };
            },
        }
        
        let name_c1 = name.clone();
        key_ordering.push(name_c1);
        offset_table.insert(name.clone(), offset);
        table.insert(name, value);
    }

    match ordering {
        None | Some(ImportOrdering::Native) => {},
        Some(ImportOrdering::Key) => {
            table.sort_unstable_keys();
        },
        Some(ImportOrdering::Offset) => {
            table.sort_by(|a,_,b,_| offset_table[a].cmp(&offset_table[b]));
        },
    }

    key_ordering.sort_by(|a,b| a.cmp(&b));

    for e in tkey_data_sorted {
        let name = string_from_name(&e.name, name_list);
        let name_c2 = name.clone();
        offset_ordering.push(name_c2);
    }
                
    return Ok(table);
}

impl GXTFile {
    //pub fn new(format: GXTFileFormat) -> GXTFile {
    //    GXTFile {
    //        format,
    //        main_table: GXTStringTable { data: IndexMap::new() },
    //        aux_tables: IndexMap::new(),
    //    }
    //}
    pub fn write_to_text (&self, file: &mut impl Write) -> Result<(),GXTError> {

        let out_string = toml::to_string(self)?;
        file.write(out_string.as_bytes())?;
        Ok(())
    }
    pub fn read_from_text (file: &mut (impl Read + std::io::Seek)) -> Result<GXTFile,GXTError> {

        let mut raw_data: String = Default::default();
        file.read_to_string(&mut raw_data)?;
        
        let file: GXTFile = toml::from_str(&raw_data)?;
        return Ok(file);
    }
    fn create_tkey(&self, table: &IndexMap<String,String>, table_name: Option<&str>, custom_table: &Option<GXTCharacterTable>) -> Result<(GXTInternalTKEY,GXTCompilationTDAT), GXTError> {

        let mut tdat = GXTCompilationTDAT {
            buffer: vec!(),
            offset_map: Default::default(),
        };

        let mut tkey = GXTInternalTKEY {
            name: match table_name {
                None => None,
                Some(s) => Some(string_to_name_basic(s)?),
            },
            offset: 0,
            size: 0,
            entries: vec!(),
        };

        for (k,v) in table {
            let offset = tdat.offset_map.get(v);
            match offset {
                Some(o) => {
                    // String exists, we reuse the existing offset
                    tkey.entries.push( GXTInternalTKEYEntry {
                        name: string_to_name(k,&self.format)?,
                        offset: *o as u32,
                    });
                },
                None => {
                    // String does not exist, we add a new one
                    let cur_pos: usize = tdat.buffer.len();
                    let _ = tdat.buffer.write(&encode_string(v,&self.format,custom_table)?);
                    
                    tkey.entries.push( GXTInternalTKEYEntry {
                        name: string_to_name(k,&self.format)?,
                        offset: cur_pos as u32,
                    });
                },
            };
            tkey.size += match self.format {
                GXTFileFormat::Three | GXTFileFormat::Vice => 12, //4 for offset, 8 for name
                GXTFileFormat::San8 | GXTFileFormat::San16 => 8, //4 for offset, 4 for CRC32
            };
        }
        match self.format {
            GXTFileFormat::San8 | GXTFileFormat::San16 => {
                // not described on gtamods.com wiki page, it seems like each successive TKEY/TDAT
                // gets aligned across a 4-byte boundary -- in practice, this just means that
                // each TDAT's length must be padded until it can divide by 4, because all the
                // other blocks already have length divisible by 4
                let filler: u32 = if (tdat.buffer.len() % 4) != 0 {
                    4 - (u32::try_from(tdat.buffer.len()).unwrap() % 4)
                } else { 0 };

                for _ in 0..filler {
                    tdat.buffer.push(0);
                }
            },
            _ => {},
        };
        Ok((tkey,tdat))
    }
    fn write_tkey_to_gxt(&self, file: &mut impl Write, tkey: &GXTInternalTKEY) -> Result<(), GXTError> {
        match tkey.name {
            None => {},
            Some(t) => {
                file.write(&t)?;
            },
        }
        file.write(b"TKEY")?;
        file.write(&u32::to_le_bytes(tkey.size))?;

        // TKEY entries MUST be sorted by key in the actual GXT file, as games seem to do a binary
        // search when retrieving strings from it
        let mut entries_sorted = tkey.entries.clone();
        entries_sorted.sort_by(|a,b| a.name.cmp(&b.name));

        for e in &entries_sorted {
            file.write(&u32::to_le_bytes(e.offset))?;
            match self.format {
                GXTFileFormat::Three | GXTFileFormat::Vice => {
                    match e.name {
                        GXTStringName::Text(t) => { file.write(&t)?; },
                        GXTStringName::CRC32(_) => { return Err(GXTError::CompilationError("File of this format cannot have CRC32-based string names".to_string())); },
                    }
                },
                GXTFileFormat::San8 | GXTFileFormat::San16 => {
                    match e.name {
                        GXTStringName::CRC32(h) => { file.write(&u32::to_le_bytes(h))?; },
                        GXTStringName::Text(_) => { return Err(GXTError::CompilationError("File of this format cannot have text-based string names".to_string())); }, // this is not an error the end user should see, as text-based names are converted to CRC32 when exporting an SA format GXT
                    }
                },
            }
        }
        Ok(())
    }
    pub fn write_to_gxt (&self, file: &mut impl Write, custom_table: &Option<GXTCharacterTable>) -> Result<(), GXTError> {

        let (main_tkey,main_tdat) = self.create_tkey(&self.main_table, None, custom_table)?;

        let mut aux_data: Vec<(GXTInternalTKEY,GXTCompilationTDAT)> = vec!();

        for (k,v) in &self.aux_tables {
            aux_data.push(self.create_tkey(&v, Some(k), custom_table)?);
        }

        match self.format {
            GXTFileFormat::Three => {
                self.write_tkey_to_gxt(file,&main_tkey)?;
                file.write(b"TDAT")?;
                file.write(&u32::to_le_bytes(main_tdat.buffer.len().try_into().unwrap()))?;
                file.write(&main_tdat.buffer)?;
                Ok(())
            },
            GXTFileFormat::Vice => {
                file.write(b"TABL")?;
                let tabl_size: u32 = 12u32 * (1 + u32::try_from(self.aux_tables.len()).unwrap());
                file.write(&u32::to_le_bytes( tabl_size ))?;

                let mut table_offset = tabl_size + 8;
                file.write(b"MAIN\0\0\0\0")?;
                file.write(&u32::to_le_bytes( table_offset ))?;
                table_offset += 8 + main_tkey.size + 8 + u32::try_from(main_tdat.buffer.len()).unwrap();

                for e in &aux_data {
                    match e.0.name {
                        Some(n) => {
                            let table_name: [u8;8] = n;
                            file.write(&table_name)?;
                            file.write(&u32::to_le_bytes( table_offset ))?;
                        },
                        None => {
                            return Err(GXTError::CompilationError("Auxiliary tables must have a name".to_string()));
                        },
                    }
                    table_offset += 16 + e.0.size + 8 + u32::try_from(e.1.buffer.len()).unwrap();
                }
                
                self.write_tkey_to_gxt(file,&main_tkey)?;
                file.write(b"TDAT")?;
                file.write(&u32::to_le_bytes(main_tdat.buffer.len().try_into().unwrap()))?;
                file.write(&main_tdat.buffer)?;
                
                for e in &aux_data {
                    self.write_tkey_to_gxt(file,&e.0)?;
                    file.write(b"TDAT")?;
                    file.write(&u32::to_le_bytes(e.1.buffer.len().try_into().unwrap()))?;
                    file.write(&e.1.buffer)?;
                }
                Ok(())
            },
            GXTFileFormat::San8 | GXTFileFormat::San16 => {
                file.write(&u16::to_le_bytes(4))?;
                file.write(&u16::to_le_bytes( match self.format {
                    GXTFileFormat::San8 => 8,
                    GXTFileFormat::San16 => 16,
                    _ => { return Err(GXTError::CompilationError("This GTA SA format is somehow not a GTA SA format?".to_string())); }
                }))?;

                file.write(b"TABL")?;
                let tabl_size: u32 = 12u32 * (1 + u32::try_from(self.aux_tables.len()).unwrap());
                file.write(&u32::to_le_bytes( tabl_size ))?;

                let mut table_offset = 4 + tabl_size + 8;
                file.write(b"MAIN\0\0\0\0")?;
                file.write(&u32::to_le_bytes( table_offset ))?;
                table_offset += 8 + main_tkey.size + 8 + u32::try_from(main_tdat.buffer.len()).unwrap();

                for e in &aux_data {
                    match e.0.name {
                        Some(n) => {
                            let table_name: [u8;8] = n;
                            file.write(&table_name)?;
                            file.write(&u32::to_le_bytes( table_offset ))?;
                        },
                        None => {
                            return Err(GXTError::CompilationError("Auxiliary tables must have a name".to_string()));
                        },
                    }
                    table_offset += 16 + e.0.size + 8 + u32::try_from(e.1.buffer.len()).unwrap();
                }
                
                self.write_tkey_to_gxt(file,&main_tkey)?;
                file.write(b"TDAT")?;
                file.write(&u32::to_le_bytes(main_tdat.buffer.len().try_into().unwrap()))?;
                file.write(&main_tdat.buffer)?;
                
                for e in &aux_data {
                    self.write_tkey_to_gxt(file,&e.0)?;
                    file.write(b"TDAT")?;
                    file.write(&u32::to_le_bytes(e.1.buffer.len().try_into().unwrap()))?;
                    file.write(&e.1.buffer)?;
                }
                Ok(())
            },
        }

    }
    pub fn read_from_gxt (file: &mut (impl Read + std::io::Seek), ordering: &Option<ImportOrdering>, custom_table: &Option<GXTCharacterTable>, name_list: &Option<HashMap<u32, String>>) -> Result<GXTFile,GXTError> {
        
        let mut first_four_bytes: [u8; 4] = [0;4];
        file.read_exact(&mut first_four_bytes)?;

        let format = if first_four_bytes == *b"TKEY" { //GTA3 format files do not have a TABL
            GXTFileFormat::Three
        } else if first_four_bytes == *b"TABL" { //VC format files do
            GXTFileFormat::Vice
        } else if first_four_bytes == *b"\x04\0\x08\0" { //SA, 8-bit characters
            GXTFileFormat::San8
        } else if first_four_bytes == *b"\x04\0\x10\0" { //SA, 16-bit characters
            GXTFileFormat::San16
        } else { 
            return Err(GXTError::ParsingError("This GXT file does not match any known GTA 3 / VC / SA format.".to_string()));
        };
        file.seek(std::io::SeekFrom::Start(0))?; //seek back to the start

        match format {
            GXTFileFormat::Three => {
                let tkey = gxt_read_tkey(file,&format,None,None,&ordering)?;
                return Ok(GXTFile {
                    main_table: {gxt_read_tdat(file, &tkey, None, &format, &ordering, custom_table, name_list)?},
                    format: format,
                    aux_tables: IndexMap::new(),
                });
            },
            GXTFileFormat::Vice | GXTFileFormat::San8 | GXTFileFormat::San16 => {
                
                match format {
                    GXTFileFormat::San8 | GXTFileFormat::San16 => {
                        let mut raw_version_number: [u8; 2] = [0;2];
                        let mut raw_character_size: [u8; 2] = [0;2];
                        file.read_exact(&mut raw_version_number)?;
                        file.read_exact(&mut raw_character_size)?;
                        let version_number = u16::from_le_bytes(raw_version_number);
                        let character_size = u16::from_le_bytes(raw_character_size);
                    
                        if version_number != 4 {return Err(GXTError::ParsingError(format!("The GXT file has version {}, must have version 4",version_number) ));}
                        match character_size {
                            8 => (),
                            16 => (),
                            _ => {return Err(GXTError::ParsingError(format!("The GXT file has character size {}, must have 8 or 16",character_size) ));}
                        }
                    },
                    _ => {},
                }

                let tabl = gxt_read_tabl(file)?;

                if !tabl.entries[0].is_main {
                    return Err(GXTError::ParsingError("GXT File error: The first table must be MAIN".to_string()));
                }

                let _tkeys: Result<Vec<GXTInternalTKEY>,_> = 
                    tabl.entries.iter().map(|k| gxt_read_tkey(
                        file,
                        &format,
                        match k.is_main { true => None, false => Some(k.name), },
                        Some(k.offset),
                        ordering
                        )).collect();
                let tkeys = _tkeys?;

                let mut _key_ordering: Vec<String> = tkeys[1..].iter().map(|k| match k.name {
                    None => "".to_string(),
                    Some(n) => string_from_name(&GXTStringName::Text(n), name_list)
                }).collect();
                let mut _offset_ordering: Vec<(String,u32)> = tkeys[1..].iter().map(|k| (match k.name {
                    None => "".to_string(),
                    Some(n) => string_from_name(&GXTStringName::Text(n), name_list)
                }, k.offset)).collect();
                _key_ordering.sort_by(|a,b| (a).cmp(&b));
                _offset_ordering.sort_by(|a,b| (a.1).cmp(&b.1));

                let mut aux_tables: IndexMap<String, IndexMap<String,String>> = IndexMap::new();
                for e in &tkeys[1..] {
                    let name_string = match e.name {
                        None => { return Err(GXTError::ParsingError("An auxiliary table must have a name!".to_string())); },
                        Some(n) => string_from_name(&GXTStringName::Text(n), name_list)
                        };

                    let new_table = gxt_read_tdat(file, &e, Some(e.offset), &format, ordering, custom_table, name_list);
                    match new_table {
                        Ok(t) => {
                            aux_tables.insert(name_string.clone(), t);
                        },
                        Err(x) => {
                            return Err(GXTError::ParsingError(format!("Error while parsing table ({}): {}",&name_string, x)));
                        },
                    };
                }

                //match ordering {
                //    None | Some(ImportOrdering::Native) => {},
                //    Some(ImportOrdering::Key) => {
                //    },
                //    Some(ImportOrdering::Offset) => {
                //    },
                //}
                
                //eprintln!("Reading main table...");
                return Ok(GXTFile {
                    main_table: gxt_read_tdat(file, &tkeys[0], Some(tkeys[0].offset), &format, ordering, custom_table, name_list)?,
                    format: format,
                    aux_tables,
                });
            },
        };
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;
    use super::*;

    #[test]
    fn gta3_compilation_test() {
            
        let _f = File::open("test_files/gta3.txt").expect("Unable to open text file");
        let mut file = BufReader::new(_f);
        let gxt = GXTFile::read_from_text(&mut file).expect("Unable to load GXT data from text file");
        
        assert!( gxt.main_table.len() == 10 );
        assert!( gxt.main_table.get("FEM_MM") == Some(&"HELLO WORLD".to_string()) );

        let mut compiled_data: Vec<u8> = vec!();
        gxt.write_to_gxt(&mut compiled_data,&None).expect("Unable to compile GXT file");

        // raw GXT file made by hand!
        let mut comparison_file = File::open("test_files/gta3.gxt").expect("Unable to open GXT file");
        let mut comparison_data: Vec<u8> = vec!();
        comparison_file.read_to_end(&mut comparison_data).expect("Unable to read test GXT value");

        assert!( compiled_data == comparison_data );
        
    }
    
    #[test]
    fn gtavc_compilation_test() {
            
        let _f = File::open("test_files/gtavc.txt").expect("Unable to open text file");
        let mut file = BufReader::new(_f);
        let gxt = GXTFile::read_from_text(&mut file).expect("Unable to load GXT data from text file");
        
        assert!( gxt.main_table.len() == 10 );
        assert!( gxt.main_table.get("FEM_MM") == Some(&"HELLO WORLD".to_string()) );

        assert!( gxt.aux_tables.len() == 1 );

        let mut compiled_data: Vec<u8> = vec!();
        gxt.write_to_gxt(&mut compiled_data,&None).expect("Unable to compile GXT file");

        // raw GXT file made by hand!
        let mut comparison_file = File::open("test_files/gtavc.gxt").expect("Unable to open GXT file");
        let mut comparison_data: Vec<u8> = vec!();
        comparison_file.read_to_end(&mut comparison_data).expect("Unable to read test GXT value");

        assert!( compiled_data == comparison_data );
        
    }
    
    #[test]
    fn gtasa_compilation_test() {
            
        let _f = File::open("test_files/gtasa.txt").expect("Unable to open text file");
        let mut file = BufReader::new(_f);
        let gxt = GXTFile::read_from_text(&mut file).expect("Unable to load GXT data from text file");
        
        assert!( gxt.main_table.len() == 10 );
        //assert!( gxt.main_table.get("FEM_MM") == Some(&"HELLO WORLD".to_string()) );

        assert!( gxt.aux_tables.len() == 1 );

        let mut compiled_data: Vec<u8> = vec!();
        gxt.write_to_gxt(&mut compiled_data,&None).expect("Unable to compile GXT file");

        // raw GXT file made by hand!
        let mut comparison_file = File::open("test_files/gtasa.gxt").expect("Unable to open GXT file");
        let mut comparison_data: Vec<u8> = vec!();
        comparison_file.read_to_end(&mut comparison_data).expect("Unable to read test GXT value");

        assert!( compiled_data == comparison_data );
        
    }
}
