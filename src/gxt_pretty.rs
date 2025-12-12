use gxter::GXTFileFormat;

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

pub fn pretty_print(string: &str, format: &GXTFileFormat) -> Result<String,String> {

    let tokens = split_into_tokens(&string)?;
    let mut output: String = Default::default();
    let default_style = anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::White.into()));
    let mut style = default_style;

    for t in tokens {
        match t {
            GXTToken::Text(s) => {
                output.push_str(&format!("{}{}",style.render(),s));
            },
            GXTToken::Tag(t) => {

                match format {
                    GXTFileFormat::Three => {
                        match t.as_str() {
                            "b" => { style = style.fg_color(Some(anstyle::AnsiColor::BrightBlue.into())); } ,
                            "g" => { style = style.fg_color(Some(anstyle::AnsiColor::Green.into())); } ,
                            "h" => { style = style.fg_color(Some(anstyle::AnsiColor::BrightWhite.into())); } ,
                            "l" => { style = style.fg_color(Some(anstyle::AnsiColor::Black.into())); } ,
                            "r" => { style = style.fg_color(Some(anstyle::AnsiColor::Red.into())); } ,
                            "w" => { output.push_str(&format!("{}",style.render_reset()));
                                style = default_style; } ,
                            "y" => { style = style.fg_color(Some(anstyle::AnsiColor::Yellow.into())); } ,
                            _ => { output.push_str(&format!("{}~{}~",style.render(),t)); },
                        }
                    },
                    GXTFileFormat::Vice => {
                        match t.as_str() {
                            "b" => { style = style.fg_color(Some(anstyle::AnsiColor::Blue.into())); } ,
                            "g" => { style = style.fg_color(Some(anstyle::AnsiColor::BrightRed.into())); } ,
                            "h" => { style = style.fg_color(Some(anstyle::AnsiColor::BrightWhite.into())); } ,
                            "l" => { output.push_str(&format!("{}",style.render_reset()));
                                style = default_style; } ,
                            "o" => { style = style.fg_color(Some(anstyle::AnsiColor::BrightMagenta.into())); } ,
                            "p" => { style = style.fg_color(Some(anstyle::AnsiColor::Magenta.into())); } ,
                            "r" => { style = style.fg_color(Some(anstyle::AnsiColor::BrightRed.into())); } ,
                            "t" => { style = style.fg_color(Some(anstyle::AnsiColor::BrightGreen.into())); } ,
                            "w" => { style = style.fg_color(Some(anstyle::AnsiColor::White.into())); } ,
                            "x" => { style = style.fg_color(Some(anstyle::AnsiColor::BrightBlue.into())); } ,
                            "y" => { style = style.fg_color(Some(anstyle::AnsiColor::BrightYellow.into())); } ,
                            _ => { output.push_str(&format!("{}~{}~",style.render(),t)); },
                        }
                    },
                    GXTFileFormat::San8 | GXTFileFormat::San16 => {
                        match t.as_str() {
                            "A" => { output.push_str("{left analog stick click}"); } ,
                            "b" => { style = style.fg_color(Some(anstyle::AnsiColor::Blue.into())); } ,
                            "K" => { output.push_str("{left trigger}"); } ,
                            "c" => { output.push_str("{right analog stick click}"); } ,
                            "d" => { output.push_str("{down on d-pad}"); } ,
                            "g" => { style = style.fg_color(Some(anstyle::AnsiColor::Green.into())); } ,
                            "h" => { style = style.fg_color(Some(anstyle::AnsiColor::BrightWhite.into())); } ,
                            "j" => { output.push_str("{right trigger}"); } ,
                            "l" => { style = style.fg_color(Some(anstyle::AnsiColor::Black.into())); } ,
                            "m" => { output.push_str("{left bumper / white button}"); } ,
                            "n" => { output.push_str("\n\t"); },
                            "o" => { output.push_str("{right face button}"); } ,
                            "p" => { style = style.fg_color(Some(anstyle::AnsiColor::Magenta.into())); } ,
                            "q" => { output.push_str("{left face button}"); } ,
                            "r" => { style = style.fg_color(Some(anstyle::AnsiColor::Red.into())); } ,
                            "s" => { output.push_str(&format!("{}",style.render_reset()));
                                style = default_style; } ,
                            "t" => { output.push_str("{top face button}"); } ,
                            "u" => { output.push_str("{up on d-pad}"); } ,
                            "v" => { output.push_str("{right bumper / black button}"); } ,
                            "w" => { style = style.fg_color(Some(anstyle::AnsiColor::White.into())); } ,
                            "x" => { output.push_str("{bottom face button}"); } ,
                            "y" => { style = style.fg_color(Some(anstyle::AnsiColor::Yellow.into())); } ,
                            "z" => { output.push_str("{subtitle}"); } ,
                            "<" => { output.push_str("{left on d-pad}"); } ,
                            ">" => { output.push_str("{right on d-pad}"); } ,
                            _ => { output.push_str(&format!("{}~{}~",style.render(),t)); },
                        }
                    },
                }
            },
        }
    }
    output.push_str(&format!("{}",style.render_reset()));
    
    return Ok(output);
}
