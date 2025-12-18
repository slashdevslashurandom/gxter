use gxter::GXTFileFormat;
use std::default::Default;
use owo_colors::{OwoColorize, Style, colors::*};

enum GXTToken {
    Text(String),
    Tag(String),
}

fn split_into_tokens(string: &str) -> Result<Vec<GXTToken>,String> {

    let mut res: Vec<GXTToken> = vec!();

    let mut current_token: String = Default::default();

    for e in string.chars().into_iter() {
        if current_token.len() == 0 {
            current_token.push(e); // we can't be choosy over
        } else {
            if current_token.chars().nth(0) == Some('~') {
                // we are currently IN a token
                if e == '~' {
                    res.push(GXTToken::Tag(current_token[1..].to_string()));
                    current_token = "".to_string();
                } else {
                    current_token.push(e);
                }
            } else {
                // we are currently NOT in a token
                if e == '~' {
                    res.push(GXTToken::Text(current_token.to_string()));
                    current_token = "~".to_string();
                } else {
                    current_token.push(e);
                }
            }
        }
    }
    res.push(GXTToken::Text(current_token.to_string()));
    return Ok(res);
}

pub fn pretty_print(name: &str, string: &str, format: &GXTFileFormat) -> Result<(),String> {

    let tokens = split_into_tokens(&string)?;
    let default_style = Style::new().white();
    let mut style = default_style;

    print!("{} = ",name);

    for t in tokens {
        match t {
            GXTToken::Text(s) => {
                print!("{}",s.style(style));
            },
            GXTToken::Tag(t) => {

                match format {
                    GXTFileFormat::Three => {
                        match t.as_str() {
                            "b" => { style = style.fg::<BrightBlue>(); } ,
                            "g" => { style = style.fg::<Green>(); } ,
                            "h" => { style = style.fg::<BrightWhite>(); } ,
                            "l" => { style = style.fg::<Black>(); } ,
                            "r" => { style = style.fg::<Red>(); } ,
                            "w" => { style = style.remove_fg(); } ,
                            "y" => { style = style.fg::<Yellow>(); } ,
                            _ => { let full_string = "~".to_owned() + &t + "~";
                                print!("{}",full_string.style(style)); },
                        }
                    },
                    GXTFileFormat::Vice => {
                        match t.as_str() {
                            "b" => { style = style.fg::<Blue>(); } ,
                            "g" => { style = style.fg::<BrightRed>(); } ,
                            "h" => { style = style.fg::<BrightWhite>(); } ,
                            "l" => { style = style.remove_fg(); } ,
                            "o" => { style = style.fg::<BrightMagenta>(); } ,
                            "p" => { style = style.fg::<Magenta>(); } ,
                            "r" => { style = style.fg::<BrightRed>(); } ,
                            "t" => { style = style.fg::<BrightGreen>(); } ,
                            "w" => { style = style.fg::<White>(); } ,
                            "x" => { style = style.fg::<BrightBlue>(); } ,
                            "y" => { style = style.fg::<BrightYellow>(); } ,
                            _ => { let full_string = "~".to_owned() + &t + "~";
                                print!("{}",full_string.style(style)); },
                        }
                    },
                    GXTFileFormat::San8 | GXTFileFormat::San16 => {
                        match t.as_str() {
                            "A" => { print!("{{left analog stick click}}"); } ,
                            "b" => { style = style.fg::<Blue>(); } ,
                            "K" => { print!("{{left trigger}}"); } ,
                            "c" => { print!("{{right analog stick click}}"); } ,
                            "d" => { print!("{{down on d-pad}}"); } ,
                            "g" => { style = style.fg::<Green>(); } ,
                            "h" => { style = style.fg::<BrightWhite>(); } ,
                            "j" => { print!("{{right trigger}}"); } ,
                            "l" => { style = style.fg::<Black>(); } ,
                            "m" => { print!("{{left bumper / white button}}"); } ,
                                    //cycle weapons left, look left in vehicle, zoom in
                            "n" => { print!("\n\t"); }, //new line
                            "o" => { print!("{{right face button}}"); } ,
                            "p" => { style = style.fg::<Magenta>(); } ,
                            "q" => { print!("{{left face button}}"); } ,
                            "r" => { style = style.fg::<Red>(); } ,
                            "s" => { style = default_style; } ,
                            "t" => { print!("{{top face button}}"); } ,
                            "u" => { print!("{{up on d-pad}}"); } ,
                            "v" => { print!("{{right bumper / black button}}"); } ,
                                    //cycle weapons right, look right in vehicle, zoom out
                            "w" => { style = style.fg::<White>(); } ,
                            "x" => { print!("{{bottom face button}}"); } ,
                            "y" => { style = style.fg::<Yellow>(); } ,
                            "z" => { print!("💬"); } , //subtitle, will be hidden if subtitles are
                                                       //disabled in the game's options
                            "<" => { print!("{{left on d-pad}}"); } ,
                            ">" => { print!("{{right on d-pad}}"); } ,
                            _ => { let full_string = "~".to_owned() + &t + "~";
                                print!("{}",full_string.style(style)); },
                        }
                    },
                }
            },
        }
    }
    println!();
    
    return Ok(());
}
