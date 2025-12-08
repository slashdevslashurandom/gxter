extern crate getopts;
use getopts::Options;
use crate::gxt::*;
use std::env; //for env::args()
mod gxt;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

enum DisplayOrdering {
    None,
    Key,
    Offset,
}

fn print_table(table: &GXTStringTable, data_ordering: &DisplayOrdering) {
        match data_ordering {
            DisplayOrdering::None => {
                for (k, v) in &table.data {
                    println!("{}={}", k.trim_matches(char::from(0)), v);
                }
            },
            DisplayOrdering::Key => {
                for key in &table.values_ordered_by_key {
                    println!("{}={}", key.trim_matches(char::from(0)), &table.data.get(key).unwrap());
                }
            },
            DisplayOrdering::Offset => {
                for key in &table.values_ordered_by_offset {
                    println!("{}={}", key.trim_matches(char::from(0)), &table.data.get(key).unwrap());
                }
            },
        }
}

fn main() {

    let mut opts = Options::new();
    opts.optflag("d","decompile","decompile a .gxt file into a text file, rather than the other way around");
    opts.optflag("K","key-sort","arrange strings in the same order as their keys");
    opts.optflag("O","offset-sort","arrange strings in the same order as their data locations");
    opts.optflag("h","help","print this help menu");

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone(); 

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => {m}
        Err(f) => {panic!("{}", f.to_string())}
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let decompile = matches.opt_present("d");
    
    let input_filename = if !matches.free.is_empty() { //if we have any non-parsed arguments
        matches.free[0].clone() //treat the first of them as a file name
    } else { //otherwise
        print_usage(&program, opts); //return an error message
        return;
    };
    
    let data_ordering = if matches.opt_present("key-sort") { 
        DisplayOrdering::Key
    } else if matches.opt_present("offset-sort") {
        DisplayOrdering::Offset
    } else {
        DisplayOrdering::None
    };
    
    if decompile {
        let gxt = GXTFile::read_from_file(&input_filename).expect("Unable to decompile GXT file");
        
        println!("#FMT: {}", match gxt.format {
            GXTFileFormat::Three => {"III"},
            GXTFileFormat::Vice => {"VC"},
            GXTFileFormat::San8 => {"SA8"},
            GXTFileFormat::San16 => {"SA16"},
            }
        );

        print_table(&gxt.main_table, &data_ordering);
        match data_ordering {
            DisplayOrdering::None => {
                for (k, v) in &gxt.aux_tables {
                    println!("[{}]",k);
                    print_table(&v, &data_ordering);
                }
            },
            DisplayOrdering::Key => {
                for k in &gxt.tables_ordered_by_key {
                    println!("[{}]",k);
                    print_table(&gxt.aux_tables[k], &data_ordering);
                }
            },
            DisplayOrdering::Offset => {
                for k in &gxt.tables_ordered_by_offset {
                    println!("[{}]",k);
                    print_table(&gxt.aux_tables[k], &data_ordering);
                }
            },
        }
    } else {
        eprintln!("Sorry, compilation not yet implemented.\n");
        //gxt_compile(input_filename, format);
    }

    //println!("Hello, world! {input_filename}");
}
