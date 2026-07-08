use core::fmt;
use std::{
    fmt::write,
    fs::{File, TryLockError::WouldBlock},
    io::{BufReader, Stdin},
};

use termion::event::Key;

#[derive(Debug)]
enum ParsedLineError {
    _BadId,
    _BadLine,
    _BadClassName,
    _BadState,
    MismatchGameId,
    BadNumber,
    BadCLIArgument,
    BadSeparator,
    NotEnoughCharacters,
}

#[derive(Debug)]
struct ParsedLine {
    id: i32,
    parent_id: Option<i32>, // -1 = not there
    parent: Option<String>, // -1 = not there
    class_name: String,
    function_name: String,
    state: Vec<String>,
    args: Vec<String>,
}

impl fmt::Display for ParsedLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n\r", self.state.join(" "))?;
        match &self.parent {
            Some(_) => {
                write!(
                    f,
                    "     {} {} {}",
                    self.id,
                    self.class_name,
                    self.state.join(" ")
                )?;
            }
            None => {
                write!(f, "{} {}", self.id, self.class_name)?;
            }
        }

        return write!(f, " {} {}", self.function_name, self.args.join(", "));
    }
}

impl fmt::Display for ParsedLineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsedLineError::MismatchGameId => write!(f, "NO ERROR HERE"),
            ParsedLineError::BadCLIArgument => {
                write!(f, "Bad CLI Argument. Please try again you d-bag")
            }
            ParsedLineError::_BadLine => write!(f, "Bad Line"),
            ParsedLineError::_BadId => write!(f, "Oh no! your id's are big time suck."),
            ParsedLineError::_BadClassName => write!(f, "Your classname was weak"),
            ParsedLineError::_BadState => write!(f, "F U Mccannch (BadState)"),
            ParsedLineError::BadNumber => write!(f, "F U Mccannch (BadState)"),
            ParsedLineError::BadSeparator => write!(f, "Unable to find separator"),
            ParsedLineError::NotEnoughCharacters => write!(f, "Not enough characters in string."),
        }
    }
}

impl From<ParseIntError> for ParsedLineError {
    fn from(_: ParseIntError) -> Self {
        return ParsedLineError::BadNumber;
    }
}

impl Error for ParsedLineError {}

#[derive(Debug)]
struct LoggerConfig {
    id: i32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let mut file: Option<&str> = None;
    let mut game_id: Option<i32> = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-f" => {
                file = Some(args[i + 1].as_str());
                i += 1;
            }
            "-i" => {
                game_id = match args[i + 1].as_str().parse::<i32>() {
                    Ok(v) => Some(v),
                    Err(_) => return Err(Box::new(ParsedLineError::BadCLIArgument)),
                };
                i += 1;
            }
            _ => {
                return Err(Box::new(ParsedLineError::BadCLIArgument));
            }
        }
        i += 1;
    }

    if file.is_some() {
        let file = File::open(file.unwrap())?;
        let reader = BufReader::new(file);
        interact_with_lines(parse_lines(game_id, reader)?)?;
    } else {
        for line in io::stdin().lock().lines() {
            print!("{}\n", line?);
        }
    }

    return Ok(());
}

fn group_parsed_line<'a>(plines: &'a Vec<ParsedLine>) -> Vec<Vec<&'a ParsedLine>> {
    let mut lines: Vec<Vec<&'a ParsedLine>> = vec![];
    if plines.len() == 0 {
        return lines;
    }

    let mut prev_line = &plines[0];
    let mut current_group: Vec<&'a ParsedLine> = vec![prev_line];

    for i in &plines[1..] {
        if prev_line.state == i.state || 1.parent.is_some() {
            current_group.push(i);
        } else {
            lines.push(current_group);
            current_group = vec![i];
        }

        prev_line = i;
    }

    if current_group.len() > 0 {
        lines.push(current_group);
    }
    return lines;
}

fn interact_with_lines(lines: Vec<ParsedLine>) -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode()?;
    let grouped_lines = group_parsed_line(&lines);
    let mut idx: = 0;

    if grouped_lines.len() == 0 {
        print!("Sorry no data chump\n");
        return Ok(());
    }

    for c in stdin.keys() {
        /*
            write!(stdout,
            "{}{}",
            termion::cursor::Goto(1, 1),
            termion::clear::CurrenLine
                .unwrap()
            )
         */
    }

    match c.unwrap() {
        Key::Char('q') => break,
        Key::Char('k') => idx = std::cmp::max(0, idx - 1),
        Key::Char('j') => idx = std::cmp::min(grouped_lines.len() as i32 - 1,idx + 1),
        _ => {}
    }

    print!("XXXX idx {} \n",idx);
        let group= &grouped_lines[idx as usize];
        let mut first = true;
    return Ok(());
}
