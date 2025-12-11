use serde::ser::{Serialize, Serializer, SerializeMap};
use std::io::prelude::*;
use thiserror::Error;
use indexmap::IndexMap;
use std::collections::HashMap;

#[derive(serde::Serialize,serde::Deserialize,Clone)]
pub enum GXTFileFormat {
    Three, //GTA 3, GTA VC Xbox
    Vice, //GTA VC, LCS, VCS
    San8, //GTA SA, IV (8-bit characters)
    San16, //GTA SA, IV (16-bit characters)
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum GXTStringName {
    Text([u8;8]),
    CRC32(u32),
}

#[derive(Error, Debug)]
pub enum GXTError {
    #[error("GXT file parsing error: {0}")]
    ParsingError(String),
    #[error("GXT file compilation error: {0}")]
    CompilationError(String),
    #[error("I/O error")]
    IOError(#[from] std::io::Error),
    #[error("TOML serialization error")]
    TOMLSerError(#[from] toml::ser::Error),
    #[error("TOML deserialization error")]
    TOMLDeError(#[from] toml::de::Error),
}

#[derive(serde::Serialize,serde::Deserialize)]
pub struct GXTFile {

    /// Specifies the format used when decompiling or compiling the GXT file.
    pub format: GXTFileFormat,

    /// Contains the "main" table. In GTA 3 files, this is the only table, whereas in GTA VC and SA
    /// files, it is the first table in the file.
    pub main_table: GXTStringTable,

    /// Contains all the "auxiliary" tables. This container must be empty when working with GTA 3
    /// files. The default ordering when decompiling a GXT file is to follow the list as
    /// specified in the TABL section.
    pub aux_tables: IndexMap<String,GXTStringTable>,
}

impl Serialize for GXTStringTable {
    fn serialize <S> (&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.data.len()))?;
        for (k, v) in &self.data {
            map.serialize_entry(k,v)?;
        }
        map.end()
    }
}

// we use a custom serializer to provide custom ordering of keys, but we don't need that for
// deserialization, so that one's derived instead
#[derive(serde::Deserialize)]
pub struct GXTStringTable {

    /// Contains all the strings in the table. The key is the string's name, the value is the
    /// string's contents. In GTA 3 and VC files, a string's name can be 8 bytes large at most. 
    /// In GTA SA format files, string names are encoded using CRC32, and string
    /// names retrieved from decompilation will be decoded as "#XXXXXXXX", where the letters X
    /// represent hexadecimal digits of the CRC32 hash. As it is a 9-byte string, it should be
    /// obvious that such a string can't be a "native" name.
    /// 
    #[serde(flatten)]
    pub data: IndexMap<String,String>,
}

// -- internal structures, not recommended for use

// these tables contain the default US/European character tables used by GTA 3, VC and SA.
// the first 32 elements are skipped in all of them. the tables largely match ASCII, but
// then add extra accented characters for EFIGS support or modify certain characters to add
// icons for PS2 controller buttons or the HUD.
//
// empty strings in this array are treated by the decode_character function as an indication
// that the character needs to be escaped using the \xAB or \uABCD notation.

const GTA3_DEFAULT_CHARACTER_TABLE: [&str; 224] = [
    " ", "!", "\"","#", "$", "%", "&", "'", "(", ")", "*", "+", ",", "-", ".", "/",
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ":", ";", "<", "=", ">", "?",
    "™", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O",
    "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "[", "\\","]", "^", "°",
    "`", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o",
    "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "❤", "◯",  "", "~",  "",
    "À", "Á", "Â", "Ä", "Æ", "Ç", "È", "É", "Ê", "Ë", "Ì", "Í", "Î", "Ï", "Ò", "Ó",
    "Ô", "Ö", "Ù", "Ú", "Û", "Ü", "ß", "à", "á", "â", "ä", "æ", "ç", "è", "é", "ê",
    "ë", "ì", "í", "î", "ï", "ò", "ó", "ô", "ö", "ù", "ú", "û", "ü", "Ñ", "ñ", "¿",
    "¡",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",
     "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",
     "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",
     "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",
     "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",
];

const VICE_DEFAULT_CHARACTER_TABLE: [&str; 224] = [
    " ", "!", "\"","#", "$", "%", "&", "'", "(", ")", "*", "+", ",", "-", ".", "/",
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ":", ";", "🛡", "=", "★", "?",
    "™", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O",
    "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "[", "\\","]", "¡", "°",
    "`", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o",
    "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "❤", "|", "}", "~",  "",
    "À", "Á", "Â", "Ä", "Æ", "Ç", "È", "É", "Ê", "Ë", "Ì", "Í", "Î", "Ï", "Ò", "Ó",
    "Ô", "Ö", "Ù", "Ú", "Û", "Ü", "ß", "à", "á", "â", "ä", "æ", "ç", "è", "é", "ê",
    "ë", "ì", "í", "î", "ï", "ò", "ó", "ô", "ö", "ù", "ú", "û", "ü", "Ñ", "ñ", "¿",
     "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",
     "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",
     "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",
     "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",
     "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",
]; //the additional letter characters are temporarily disabled, until i can figure out how to
   //separate them from the regular ones

const SAN_DEFAULT_CHARACTER_TABLE: [&str; 224] = [ //this is just the CP1252 codepage
    " ", "!", "\"","#", "$", "%", "&", "'", "(", ")", "*", "+", ",", "-", ".", "/",
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ":", ";", "<", "=", ">", "?",
    "@", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O",
    "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "[", "\\","]", "^", "_",
    "`", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o",
    "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "{", "|", "}", "~",  "",
    "€",  "", "‚", "ƒ", "„", "…", "†", "‡", "ˆ", "‰", "Š", "‹", "Œ",  "", "Ž",  "",
     "", "‘", "’", "“", "”", "•", "–", "—", "˜", "™", "š", "›", "œ",  "", "ž", "Ÿ",
    " ", "¡", "¢", "£", "¤", "¥", "¦", "§", "¨", "©", "ª", "«", "¬", "­", "®", "¯",
    "°", "±", "²", "³", "´", "µ", "¶", "·", "¸", "¹", "º", "»", "¼", "½", "¾", "¿",
    "À", "Á", "Â", "Ã", "Ä", "Å", "Æ", "Ç", "È", "É", "Ê", "Ë", "Ì", "Í", "Î", "Ï",
    "Ð", "Ñ", "Ò", "Ó", "Ô", "Õ", "Ö", "×", "Ø", "Ù", "Ú", "Û", "Ü", "Ý", "Þ", "ß",
    "à", "á", "â", "ã", "ä", "å", "æ", "ç", "è", "é", "ê", "ë", "ì", "í", "î", "ï",
    "ð", "ñ", "ò", "ó", "ô", "õ", "ö", "÷", "ø", "ù", "ú", "û", "ü", "ý", "þ", "ÿ",
];

fn decode_character(character_value: u16, format: &GXTFileFormat) -> String {

    let character_table: [&str; 224] = match format {
        GXTFileFormat::Three => GTA3_DEFAULT_CHARACTER_TABLE,
        GXTFileFormat::Vice => VICE_DEFAULT_CHARACTER_TABLE,
        GXTFileFormat::San8 | GXTFileFormat::San16 => SAN_DEFAULT_CHARACTER_TABLE,
    };

    if (character_value < 32) || (usize::from(character_value) >= 32 + character_table.len()) {
        let escaped = format!("\\x{character_value:04x}"); //replace unavailable characters
        return escaped;
    } else {
        let character = character_table[usize::from(character_value) - 32];
        if (character == "\\") || (character.len() == 0) {
            let escaped = format!("\\u{character_value:04x}"); //replace unavailable characters
            return escaped;
        } else {
            return character.to_string();
        }
    }
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

// returns a sanitized string name from a raw 8-byte token name
fn string_from_name(name: &GXTStringName) -> String {

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
                        let c = t[i];
                        
                        // =, [ and ] are escaped in order to avoid collisions with formatting
                        if (c >= b' ') && (c < 127) && (c != b'=') && (c != b'[') && (c != b']') {
                            ret.push(c as char);
                        } else if c == b'\\' {
                            ret.push_str("\\\\");
                        } else {
                            ret.push_str(&format!("\\x{:02x}", c));
                        }
                    }
                    return ret;
                },
            };
        },
        GXTStringName::CRC32(c) => {
            return format!("#{c:08X}");
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

    //eprintln!("Reading the {}...", match name {
    //    None => "main table".to_string(),
    //    Some(t) => "auxiliary table ".to_owned() + &string_from_name(&t),
    //});

    file.seek(std::io::SeekFrom::Start(offset.unwrap_or(0).into()))?;
    //eprintln!("Current position: {:0x}",file.stream_position()?);

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

fn gxt_read_tdat(file: &mut (impl Read + std::io::Seek), tkey: &GXTInternalTKEY, tkey_offset: Option<u32>, format: &GXTFileFormat, ordering: &Option<ImportOrdering>) -> Result<GXTStringTable,GXTError> {
    
    let mut tkey_data_sorted = tkey.entries.clone();
    tkey_data_sorted.sort_by(|a,b| a.offset.cmp(&b.offset));

    let mut key_ordering:  Vec<String> = Vec::new();
    let mut offset_ordering: Vec<String> = Vec::new();

    let tdat_offset = tkey_offset.unwrap_or(0) + tkey.size + 8 + match tkey.name {
        None => 0, //MAIN block doesn't have the extra 8 bytes at the start
        Some(_) => 8}; //named blocks do

    //eprintln!("TDAT offset is {tdat_offset:0x}");
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
        let name = string_from_name(&e.name);
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
                    value.push_str(&decode_character(character_value,&format));
                };
            },
            GXTFileFormat::San8 => {
                let mut raw_byte: [u8; 1] = [0];
                loop {
                    file.read_exact(&mut raw_byte)?;
                    if raw_byte[0] == 0 { break; }
                    value.push_str(&decode_character(raw_byte[0].into(),&format));
                };
            },
            GXTFileFormat::San16 => {
                let mut raw_2byte_sequence: [u8; 2] = [0;2];

                loop {
                    file.read_exact(&mut raw_2byte_sequence)?;
                    let character_value = raw_2byte_sequence[0] as u16;
                    if character_value == 0 { break; }
                    value.push_str(&decode_character(character_value,&format));
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
        let name = string_from_name(&e.name);
        let name_c2 = name.clone();
        offset_ordering.push(name_c2);
    }
                
    return Ok(GXTStringTable {
        data: table,
    });
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
    pub fn write_to_gxt (&self, _file: &mut impl Write) -> Result<(), GXTError> {
        Err(GXTError::CompilationError("Not implemented yet".to_string()))
    }
    pub fn read_from_gxt (file: &mut (impl Read + std::io::Seek), ordering: &Option<ImportOrdering>) -> Result<GXTFile,GXTError> {
        
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
                    main_table: {gxt_read_tdat(file, &tkey, None, &format,&ordering)?},
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
                    Some(n) => string_from_name(&GXTStringName::Text(n))
                }).collect();
                let mut _offset_ordering: Vec<(String,u32)> = tkeys[1..].iter().map(|k| (match k.name {
                    None => "".to_string(),
                    Some(n) => string_from_name(&GXTStringName::Text(n))
                }, k.offset)).collect();
                _key_ordering.sort_by(|a,b| (a).cmp(&b));
                _offset_ordering.sort_by(|a,b| (a.1).cmp(&b.1));

                let mut aux_tables: IndexMap<String, GXTStringTable> = IndexMap::new();
                for e in &tkeys[1..] {
                    let name_string = match e.name {
                        None => { return Err(GXTError::ParsingError("An auxiliary table must have a name!".to_string())); },
                        Some(n) => string_from_name(&GXTStringName::Text(n))
                        };

                    let new_table = gxt_read_tdat(file, &e, Some(e.offset), &format, ordering);
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
                    main_table: gxt_read_tdat(file, &tkeys[0], Some(tkeys[0].offset), &format, ordering)?,
                    format: format,
                    aux_tables,
                });
            },
        };
    }
}
