use io::BufReader;
use std::convert::From;
use std::env;
use std::fmt::{self, Debug};
use std::io::{self, BufRead};
use std::num::ParseIntError;
use std::{error::Error, fs::File, str::FromStr};

use std::io::Write;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

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
    parent: Option<String>,
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

        write!(f, " {} {}", self.function_name, self.args.join(", "))
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
        ParsedLineError::BadNumber
    }
}

impl Error for ParsedLineError {}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let mut file: Option<&str> = None;
    let mut game_id: Option<i32> = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-f" => {
                if i + 1 >= args.len() {
                    return Err(Box::new(ParsedLineError::BadCLIArgument));
                }
                file = Some(args[i + 1].as_str());
                i += 1;
            }
            "-i" => {
                if i + 1 >= args.len() {
                    return Err(Box::new(ParsedLineError::BadCLIArgument));
                }
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

    if let Some(file_path) = file {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        interact_with_lines(parse_lines(game_id, reader)?)?;
    } else {
        for line in io::stdin().lock().lines() {
            println!("{}", line?);
        }
    }

    Ok(())
}

fn group_parsed_line<'a>(plines: &'a [ParsedLine]) -> Vec<Vec<&'a ParsedLine>> {
    let mut lines: Vec<Vec<&'a ParsedLine>> = vec![];
    if plines.is_empty() {
        return lines;
    }

    let mut prev_line = &plines[0];
    let mut current_group: Vec<&'a ParsedLine> = vec![prev_line];

    for i in &plines[1..] {
        if prev_line.state == i.state || i.parent.is_some() {
            current_group.push(i);
        } else {
            lines.push(current_group);
            current_group = vec![i];
        }

        prev_line = i;
    }

    if !current_group.is_empty() {
        lines.push(current_group);
    }
    lines
}

fn interact_with_lines(lines: Vec<ParsedLine>) -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout().into_raw_mode()?;
    let stdin = io::stdin();
    let mut idx: i32 = 0;
    let grouped_lines = group_parsed_line(&lines);

    if grouped_lines.is_empty() {
        println!("Sorry, no data chump");
        return Ok(());
    }

    for c in stdin.keys() {
        /* yaa this is a hack
        write!(stdout,
        "{}{}",
        termion::cursor::Goto(1, 1),
        termion::clear::CurrentLine)
        .unwrap();
        */

        match c.unwrap() {
            Key::Char('q') => break,
            Key::Char('k') => idx = std::cmp::max(0, idx - 1),
            Key::Char('j') => idx = std::cmp::min(grouped_lines.len() as i32 - 1, idx + 1),
            _ => {}
        }

        println!("XXXX idx {}", idx);
        let group = &grouped_lines[idx as usize];
        let mut first = true;
        for item in group {
            if first {
                first = false;
                print!("{}\n\r", item.state.join(" "));
            }
            match &item.parent {
                Some(_) => {
                    print!(
                        "     {} {} {}",
                        item.id,
                        item.class_name,
                        item.state.join(" ")
                    );
                }
                None => {
                    print!("{} {}", item.id, item.class_name);
                }
            }
            print!(" {} {}\n\r", item.function_name, item.args.join(", "));
        }

        stdout.flush()?;
    }
    Ok(())
}

fn parse_lines<T: std::io::BufRead>(
    game_id: Option<i32>,
    item: T,
) -> Result<Vec<ParsedLine>, Box<dyn Error>> {
    let mut plines: Vec<ParsedLine> = vec![];

    for line in item.lines() {
        let line = line?;

        match parse(game_id, line.as_str()) {
            Ok(v) => {
                println!("Class name insertion (gentle baby) {}", v.class_name);
                plines.push(v);
            }

            Err(ParsedLineError::MismatchGameId) => {}

            Err(e) => {
                println!("I have found an err, and i have a head ache {:?}", e);
            }
        }
    }

    Ok(plines)
}

fn parse(game_id: Option<i32>, og_line: &str) -> Result<ParsedLine, ParsedLineError> {
    // <TS> <ID> <Class> <function name> <state args> <arguments to function>
    // Parent:<ID>:Child
    // state args
    // <timestamp> <count>:<length>:<object><length>:<object>... count <whitespace separator>
    let (_, line) = parse_number::<u64>(og_line)?;
    let line = pop_separator(line, ' ')?;

    let (id, line) = parse_number::<i32>(line)?;
    let line = pop_separator(line, ' ')?;

    let (class, line) = parse_class_name(line)?;
    let line = pop_separator(line, ' ')?;

    if let Some(gid) = game_id
        && match class.parent_id {
            Some(v) => gid != v,
            None => gid != id,
        }
    {
        return Err(ParsedLineError::MismatchGameId);
    }

    let (function_name, line) = take_until_whitespace(line)?;
    let line = pop_separator(line, ' ')?;

    let (states, line) = parse_state(line)?;
    let line = pop_separator(line, ' ')?;

    let (args, line) = parse_state(line)?;

    assert!(
        line.is_empty(),
        "I expected line to be 0 but got {}: with contents {}",
        line.len(),
        line
    );

    Ok(ParsedLine {
        function_name: function_name.to_string(),
        class_name: class.class_name.to_string(),
        id,
        parent: class.parent_class_name,
        state: states.iter().map(|x| x.to_string()).collect(),
        args: args.iter().map(|x| x.to_string()).collect(),
    })
}

#[derive(Debug)]
struct ParsedClassName {
    class_name: String,
    parent_id: Option<i32>,
    parent_class_name: Option<String>,
}

fn parse_state(line: &str) -> Result<(Vec<&str>, &str), ParsedLineError> {
    let (state_var_count, remaining) = parse_number::<i32>(line)?;

    let remaining = pop_separator(remaining, ' ')?;
    let mut states: Vec<&str> = Vec::with_capacity(state_var_count as usize);
    let mut line_consumed: usize = line.len() - remaining.len();

    for _ in 0..state_var_count {
        let (state_var_length, remaining) = parse_number::<usize>(&line[line_consumed..])?;

        let remaining = pop_separator(remaining, ':')?;
        let (state, remaining) = take_n_characters(remaining, state_var_length)?;

        states.push(state);
        line_consumed = line.len() - remaining.len();
    }
    Ok((states, &line[line_consumed..]))
}

fn parse_class_name(line: &str) -> Result<(ParsedClassName, &str), ParsedLineError> {
    let (class_name, rest_of_string) = take_until_whitespace(line)?;

    if !class_name.contains(":") {
        return Ok((
            ParsedClassName {
                class_name: class_name.to_string(),
                parent_id: None,
                parent_class_name: None,
            },
            rest_of_string,
        ));
    }

    let (parent_class, class_name) = take_until(class_name, ':')?;
    let rest_of_class_name = pop_separator(class_name, ':')?;
    let (parent_id, rest_of_class_name) = parse_number::<i32>(rest_of_class_name)?;

    let (_, rest_of_class_name) = take_until(rest_of_class_name, ':')?;
    let (class_name, _) = take_until_whitespace(rest_of_class_name)?;

    Ok((
        ParsedClassName {
            class_name: class_name.to_string(),
            parent_id: Some(parent_id),
            parent_class_name: Some(parent_class.to_string()),
        },
        rest_of_string,
    ))
}

fn parse_number<T>(string: &str) -> Result<(T, &str), ParsedLineError>
where
    T: FromStr,
    ParsedLineError: From<T::Err>,
{
    let num_count = string.chars().take_while(|c| c.is_numeric()).count();

    let value = string[..num_count]
        .parse::<T>()
        .map_err(ParsedLineError::from)?;

    Ok((value, &string[num_count..]))
}

fn pop_separator(string: &str, separator: char) -> Result<&str, ParsedLineError> {
    let count = string
        .chars()
        .take_while(|c| {
            c == &separator
        })
        .count();

    if count == 0 {
        return Err(ParsedLineError::BadSeparator);
    }

    Ok(&string[1..])
}

fn take_until_whitespace(string: &str) -> Result<(&str, &str), ParsedLineError> {
    let count = string
        .chars()
        .take_while(|c| {
            !c.is_whitespace()
        })
        .count();

    Ok((&string[..count], &string[count..]))
}

fn take_n_characters(string: &str, n: usize) -> Result<(&str, &str), ParsedLineError> {
    if string.len() < n {
        return Err(ParsedLineError::NotEnoughCharacters);
    }

    Ok((&string[..n], &string[n..]))
}

fn take_until(string: &str, character: char) -> Result<(&str, &str), ParsedLineError> {
    let count = string
        .chars()
        .take_while(|c| {
            c != &character
        })
        .count();

    Ok((&string[0..count], &string[count..]))
}
