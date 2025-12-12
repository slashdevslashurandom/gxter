extern crate getopts;
use gxter::GXTFile;
use std::io;
use std::fs::File;
use std::io::BufReader;
use getopts::Options;
use std::env; //for env::args()
mod gxt_pretty;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {

    let mut opts = Options::new();
    opts.optflag("d","decompile","decompile a .gxt file into a text file, rather than the other way around");
    opts.optflag("p","pretty-print","print the contents of a GXT or text file with color formatting");
    opts.optopt("o","output","output file name","NAME");
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
    let do_pretty_print = matches.opt_present("p");
    
    let input_filename = if !matches.free.is_empty() { //if we have any non-parsed arguments
        matches.free[0].clone() //treat the first of them as a file name
    } else { //otherwise
        print_usage(&program, opts); //return an error message
        return;
    };
   
    let data_ordering = if matches.opt_present("key-sort") { 
        gxter::ImportOrdering::Key
    } else if matches.opt_present("offset-sort") {
        gxter::ImportOrdering::Offset
    } else {
        gxter::ImportOrdering::Native
    };
    
    if do_pretty_print {
        let gxt = if decompile {
            let _f = File::open(&input_filename).expect("Unable to open GXT file");
            let mut file = BufReader::new(_f);

            GXTFile::read_from_gxt(&mut file, &Some(data_ordering)).expect("Unable to decompile GXT file")
        } else {
            let _f = File::open(&input_filename).expect("Unable to open text file");
            let mut file = BufReader::new(_f);

            GXTFile::read_from_text(&mut file).expect("Unable to decompile GXT file")
        };

        for (k,v) in gxt.main_table {
            println!("{} = {}",k,gxt_pretty::pretty_print(&v,&gxt.format).unwrap());
        }

        for (k,v) in gxt.aux_tables {
            println!("[{k}]");
            for (k,v) in v {
                println!("{} = {}",k,gxt_pretty::pretty_print(&v,&gxt.format).unwrap());
            }
            println!("");
        }
        
    } else if decompile {

        let _f = File::open(&input_filename).expect("Unable to open GXT file");
        let mut file = BufReader::new(_f);

        let gxt = GXTFile::read_from_gxt(&mut file, &Some(data_ordering)).expect("Unable to decompile GXT file");
        
        let output = matches.opt_str("o");
        match output {
            Some(ofn) => {
                let mut outfile = File::create(ofn).expect("Unable to open output file");
                gxt.write_to_text(&mut outfile).unwrap();
            },
            None => {
                let mut stdout = io::stdout();
                gxt.write_to_text(&mut stdout).unwrap();
            }
        }
        
    } else {

        let output = matches.opt_str("o");

        match output {
            Some(ofn) => {
                let _f = File::open(&input_filename).expect("Unable to open text file");
                let mut file = BufReader::new(_f);

                let gxt = GXTFile::read_from_text(&mut file).expect("Unable to decompile GXT file");

                let mut outfile = File::create(ofn).expect("Unable to open output file");
                gxt.write_to_gxt(&mut outfile).unwrap();
            },
            None => {
                eprintln!("No output file name specified!");
            },
        }
    }
}
