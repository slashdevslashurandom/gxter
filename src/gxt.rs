use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::collections::HashMap;

#[derive(Clone)]
pub enum GXTFileFormat {
    Three, //GTA 3, GTA VC Xbox
    Vice, //GTA VC, LCS, VCS
    San8, //GTA SA, IV (8-bit characters)
    San16, //GTA SA, IV (16-bit characters)
}

#[derive(Clone)]
pub enum GXTStringName {
    Text([u8;8]),
    CRC32(u32),
}

pub struct GXTFile {
    pub format: GXTFileFormat,
    pub main_table: GXTStringTable,
    pub aux_tables: HashMap<String,GXTStringTable>,
    pub tables_ordered_by_key: Vec<String>,
    pub tables_ordered_by_offset: Vec<String>,
}

pub struct GXTStringTable {
    //pub name: Option<String>,
    pub data: HashMap<String,String>,
    pub values_ordered_by_key: Vec<String>, //keys ordered by how they were arranged in TKEY
    pub values_ordered_by_offset: Vec<String>, //keys ordered by how they were arranged in TDAT
}

// -- internal structures, not recommended for use

// these tables contain the default US/European character tables used by GTA 3, VC and SA.
// the first 32 elements are skipped in all of them. the tables largely match ASCII, but
// then add extra accented characters for EFIGS support or modify certain characters to add
// icons for PS2 controller buttons or the HUD.
//
// empty strings in this array are treated by the decode_character function as an indication
// that the character needs to be escaped using the \xABCD notation.

const GTA3_DEFAULT_CHARACTER_TABLE: [&str; 224] = [
    " ", "!", "\"","#", "$", "%", "&", "'", "(", ")", "*", "+", ",", "-", ".", "/",
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ":", ";", "<", "=", ">", "?",
    "™", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O",
    "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "[", "\\","]", "^", "°",
    "`", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o",
    "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "❤", "◯", "", "~", "",
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
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ":", "a", "b", "c", "d", "e",
    "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u",
    "v", "w", "x", "y", "z", "à", "á", "â", "ä", "æ", "ç", "è", "é", "ê", "ë", "ì",
    "í", "î", "ï", "ò", "ó", "ô", "ö", "ù", "ú", "û", "ü", "ß", "ñ", "¿", "'", ".",
     "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",  "",
];

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

//struct GXTInternalTDAT {
//    size: u32, //number of elements
//    entries: Vec<Vec<u16>>, //array of UTF-16 strings
//}

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
                        
                        if (c >= b' ') && (c < 127) && (c != b'[') && (c != b']') {
                            ret.push(c as char);
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

fn gxt_read_tabl(file: &mut (impl Read + std::io::Seek)) -> Result<GXTInternalTABL,std::io::Error> {

    let mut magic_number: [u8; 4] = [0;4];
    file.read_exact(&mut magic_number)?;
    
    if magic_number != *b"TABL" {
        return Err(std::io::Error::other("Invalid TABL header"));
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

fn gxt_read_tkey(file: &mut (impl Read + std::io::Seek), format: &GXTFileFormat, name: Option<[u8;8]>, offset:Option<u32>) -> Result<GXTInternalTKEY,std::io::Error> {
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
        return Err(std::io::Error::other("Invalid TKEY header"));
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
    return Ok(tkey);
}

fn gxt_read_tdat(file: &mut (impl Read + std::io::Seek), tkey: &GXTInternalTKEY, tkey_offset: Option<u32>, format: &GXTFileFormat) -> Result<GXTStringTable,std::io::Error> {
    
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
        return Err(std::io::Error::other("Invalid TDAT header"));
    }

    let mut raw_size: [u8; 4] = [0;4];
    file.read_exact(&mut raw_size)?;

    let mut main_table = HashMap::<String,String>::new();

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
        main_table.insert(name, value);
    }

    for e in tkey_data_sorted {
        let name = string_from_name(&e.name);
        let name_c2 = name.clone();
        offset_ordering.push(name_c2);
    }
    
    return Ok(GXTStringTable {
        data: main_table,
        values_ordered_by_key: key_ordering,
        values_ordered_by_offset: offset_ordering
    });
}

impl GXTFile {
    pub fn new(format: GXTFileFormat) -> GXTFile {
        GXTFile {
            format,
            main_table: GXTStringTable { data: HashMap::new(), values_ordered_by_key: vec!(), values_ordered_by_offset: vec!() },
            aux_tables: HashMap::new(),
            tables_ordered_by_key: vec!(),
            tables_ordered_by_offset: vec!(),
        }
    }
    pub fn read_from_file (filename: &str) -> Result<GXTFile,std::io::Error> {
        let f = File::open(filename)?;
        let mut reader = BufReader::new(f);
        
        let mut first_four_bytes: [u8; 4] = [0;4];
        reader.read_exact(&mut first_four_bytes)?;

        let format = if first_four_bytes == *b"TKEY" { //GTA3 format files do not have a TABL
            GXTFileFormat::Three
        } else if first_four_bytes == *b"TABL" { //VC format files do
            GXTFileFormat::Vice
        } else if first_four_bytes == *b"\x04\0\x08\0" { //SA, 8-bit characters
            GXTFileFormat::San8
        } else if first_four_bytes == *b"\x04\0\x10\0" { //SA, 16-bit characters
            GXTFileFormat::San16
        } else { 
            return Err(std::io::Error::other("This GXT file does not match any known GTA 3 / VC / SA format."));
        };
        reader.seek(std::io::SeekFrom::Start(0))?; //seek back to the start

        match format {
            GXTFileFormat::Three => {
                let tkey = gxt_read_tkey(&mut reader,&format,None,None)?;
                return Ok(GXTFile {
                    main_table: {gxt_read_tdat(&mut reader, &tkey, None, &format)?},
                    format: format,
                    aux_tables: HashMap::new(),
                    tables_ordered_by_key: vec!(),
                    tables_ordered_by_offset: vec!(),
                });
            },
            GXTFileFormat::Vice | GXTFileFormat::San8 | GXTFileFormat::San16 => {
                
                match format {
                    GXTFileFormat::San8 | GXTFileFormat::San16 => {
                        let mut raw_version_number: [u8; 2] = [0;2];
                        let mut raw_character_size: [u8; 2] = [0;2];
                        reader.read_exact(&mut raw_version_number)?;
                        reader.read_exact(&mut raw_character_size)?;
                        let version_number = u16::from_le_bytes(raw_version_number);
                        let character_size = u16::from_le_bytes(raw_character_size);
                    
                        if version_number != 4 {return Err(std::io::Error::other(format!("The GXT file has version {}, must have version 4",version_number) ));}
                        match character_size {
                            8 => (),
                            16 => (),
                            _ => {return Err(std::io::Error::other(format!("The GXT file has character size {}, must have 8 or 16",character_size) ));}
                        }
                    },
                    _ => {},
                }

                let tabl = gxt_read_tabl(&mut reader)?;

                if !tabl.entries[0].is_main {
                    return Err(std::io::Error::other("GXT File error: The first table must be MAIN"));
                }

                let _tkeys: Result<Vec<GXTInternalTKEY>,_> = 
                    tabl.entries.iter().map(|k| gxt_read_tkey(
                        &mut reader,
                        &format,
                        match k.is_main { true => None, false => Some(k.name), },
                        Some(k.offset)
                        )).collect();
                let tkeys = _tkeys?;

                let tables_ordered_by_key: Vec<String> = tkeys[1..].iter().map(|k| match k.name {
                    None => "".to_string(),
                    Some(n) => string_from_name(&GXTStringName::Text(n))
                }).collect();
                let mut _offset_ordering: Vec<(String,u32)> = tkeys[1..].iter().map(|k| (match k.name {
                    None => "".to_string(),
                    Some(n) => string_from_name(&GXTStringName::Text(n))
                }, k.offset)).collect();
                _offset_ordering.sort_by(|a,b| (a.1).cmp(&b.1));
                let tables_ordered_by_offset: Vec<String> = _offset_ordering.into_iter().map(|a| a.0).collect();

                let mut aux_tables: HashMap<String, GXTStringTable> = HashMap::new();
                for e in &tkeys[1..] {
                    let name_string = match e.name {
                        None => { return Err(std::io::Error::other("GXT File error: An auxiliary table must have a name!")); },
                        Some(n) => string_from_name(&GXTStringName::Text(n))
                        };

                    let new_table = gxt_read_tdat(&mut reader, &e, Some(e.offset), &format);
                    match new_table {
                        Ok(t) => {
                            aux_tables.insert(name_string.clone(), t);
                        },
                        Err(x) => {
                            return Err(std::io::Error::other(format!("Error while parsing table ({}): {}",&name_string, x)));
                        },
                    };
                }
                
                //eprintln!("Reading main table...");
                return Ok(GXTFile {
                    main_table: gxt_read_tdat(&mut reader, &tkeys[0], Some(tkeys[0].offset), &format)?,
                    format: format,
                    aux_tables,
                    tables_ordered_by_key,
                    tables_ordered_by_offset,
                });
            },
        };
    }
}
