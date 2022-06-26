use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
    ops::{Deref, DerefMut},
    str::FromStr,
};

const VERSION: &str = "0.0.5";

#[derive(Debug)]
pub struct IError {
    error: String,
}
impl IError {
    pub fn message<T>(error: T) -> Box<Self>
    where
        T: ToString,
    {
        Box::new(Self {
            error: error.to_string(),
        })
    }
}
impl Display for IError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}
impl Error for IError {}

#[derive(Default, Debug)]
pub struct Stack {
    vec: Vec<u32>,
}
impl Deref for Stack {
    type Target = Vec<u32>;
    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}
impl DerefMut for Stack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}
impl Stack {
    pub fn top(&self) -> Result<u32, Box<dyn Error>> {
        if !self.vec.is_empty() {
            Ok(self.vec[self.len() - 1])
        } else {
            Err(IError::message(
                "trying to access the stack while it is empty",
            ))
        }
    }
}
pub struct Interpreter {
    memory: [u32; 131072],
    stack: Stack,
    lines: Vec<String>,
    opcode_params: [u8; 14],
    names: HashMap<String, usize>,
    pos: usize,
    debug: bool,
}

impl Interpreter {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = OpenOptions::new().read(true).open(path)?;

        let buff_reader = BufReader::new(&file);
        let lines = buff_reader
            .lines()
            .flat_map(|a| a.ok())
            .collect::<Vec<String>>();
        Ok(Self {
            memory: [0; 131072],
            stack: Stack::default(),
            lines,
            opcode_params: [0, 2, 2, 1, 3, 1, 3, 0, 0, 1, 1, 1, 1, 0],
            pos: 0,
            debug: false,
            names: HashMap::new(),
        })
    }
    pub fn debug(&mut self, new: bool) {
        self.debug = new;
    }
    pub fn remove_comments(string: &str) -> &str {
        let mut res = string;

        let comments = string.split(";").collect::<Vec<&str>>();
        if !comments.is_empty() {
            res = comments[0].trim();
        }
        res
    }
    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let mut last_opcode = 0;
        loop {
            if self.verify_index_overflow(self.pos) {
                break;
            }
            let pre_string = self.lines[self.pos].to_owned();
            let mut string = pre_string.trim();

            string = Self::remove_comments(string);

            // skip aliases
            if string.contains(':') {
                self.pos += 1;
                continue;
            }

            //skip spaces
            let spaces: Vec<&str> = string.split(' ').collect();
            if spaces.is_empty()
                || (spaces.len() == 1 && spaces[0] != "\n")
            {
                if string != "robson" {
                    self.pos += 1;
                    continue;
                }
            }

            // Implements the push abreviation
            if last_opcode == 3
                && !string.contains("robson")
                && !string.contains(":")
            {
                self.command(3, string, "", "")?;
                self.pos += 1;
                continue;
            }

            //get params and opcodes
            let mut opcode: u8 = 0;
            let mut params: [String; 3] =
                ["".to_owned(), "".to_owned(), "".to_owned()];

            for i in spaces {
                if i != "robson" {
                    return Err(IError::message(format!(
                        "invalid token for opcode in line {}, '{}'",
                        self.pos + 1,
                        i
                    )));
                }
                opcode += 1;
            }

            let param_count = self.opcode_params[opcode as usize];
            for i in 0..param_count {
                self.pos += 1;
                if self.verify_index_overflow(self.pos) {
                    return Err(IError::message(format!(
                        "missing params command of line {}",
                        self.pos - i as usize,
                    )));
                }
                let string = self.lines[self.pos].to_owned();
                if string.trim().len() < 2 {
                    return Err(IError::message(format!(
                        "missing params command of line {}",
                        self.pos - i as usize,
                    )));
                }
                params[i as usize] = string;
            }

            //update and run command
            self.pos += 1;
            if self.debug {
                println!("\nopcode {}", opcode);
                println!("count: {}", param_count);
                println!("params: {}, {}", params[0], params[1]);
                println!("string '{}'", string);
                println!("stack {:?}", self.stack.vec);
            }
            last_opcode = opcode;
            self.command(opcode, &params[0], &params[1], &params[2])?;
        }
        Ok(())
    }
    fn command(
        &mut self,
        opcode: u8,
        param1: &str,
        param2: &str,
        param3: &str,
    ) -> Result<(), Box<dyn Error>> {
        match opcode {
            //ADD TO TWO VALUES
            1 => {
                let value: u32 = self.get_real_value(param1)?;
                let value2: u32 = self.get_real_value(param2)?;
                self.stack.push(value + value2);
            }

            //SUBTRACT TWO VALUES
            2 => {
                let value: u32 = self.get_real_value(param1)?;
                let value2: u32 = self.get_real_value(param2)?;
                self.stack.push(value - value2);
            }

            //PUSH TO STACK SOME VALUE
            3 => {
                let value = self.get_real_value(param1)?;
                self.stack.push(value);
            }
            //IF TRUE JUMP
            4 => {
                let value: u32 = self.get_real_value(param1)?;
                let value2: u32 = self.get_real_value(param2)?;
                let pos: u32 = self.get_real_value(param3)?;
                if value == value2 {
                    self.pos = (pos - 1) as usize;
                }
            }
            //VERIFY THE STACK IF IS EMPTY JUMP
            5 => {
                let value: u32 = self.get_real_value(param1)?;
                if self.stack.is_empty() {
                    self.pos = (value - 1) as usize;
                }
            }
            //GET INPUT AND SET TO A ADDRESS
            6 => {
                let mut value = self.get_real_value::<u32>(param1)?;
                let r#type = self.get_real_value::<u32>(param2)?;
                let limit = self.get_real_value::<u32>(param3)?;
                let mut buff = String::new();
                std::io::stdout().flush()?;
                std::io::stdin().read_line(&mut buff)?;
                match r#type {
                    1 => {
                        self.memory[value as usize] =
                            buff.trim().parse::<u32>()?
                    }
                    _ => {
                        for (i, char) in buff.chars().enumerate() {
                            if i < limit as usize {
                                let char =
                                    if char == '\n' { '\0' } else { char };
                                self.memory[value as usize] = char as u32;
                                value += 1;
                            } else {
                                break;
                            }
                        }
                        self.memory[value as usize] = 0;
                    }
                }
            }

            //PRINT THE LAST AS ASCII
            7 => {
                if self.stack.is_empty() {
                    return Err(IError::message(format!(
                        "trying to use the stack while empty at line {}",
                        self.pos
                    )));
                }
                print!("{}", (self.stack.top()? as u8) as char);
                self.stack.pop();
            }

            //PRINT LAST AS NUMBER
            8 => {
                if self.stack.is_empty() {
                    return Err(IError::message(format!(
                        "trying to use the stack while empty at line {}",
                        self.pos
                    )));
                }
                print!("{}", self.stack.top()?);
                self.stack.pop();
            }

            //JUMP
            9 => {
                let value = self.get_real_value::<u32>(param1)?;
                self.pos = (value - 1) as usize;
            }

            //SET TO MEMEORY
            10 => {
                let address = self.get_real_value::<u32>(param1)? as usize;
                let value = self.stack.top()?;
                self.stack.pop();
                self.memory[address] = value as u32;
            }
            //POP STACK
            11 => {
                if !self.stack.is_empty() {
                    self.stack.pop();
                }
            }

            //GET ALL THE STRING BUFFER
            12 => {
                let mut value = self.get_real_value::<u32>(param1)?;
                let mut buffer: Vec<u32> = Vec::new();

                loop {
                    let temp = self.memory[value as usize];
                    if temp != 0 {
                        buffer.push(temp);
                        value += 1;
                    } else {
                        break;
                    }
                }
                buffer.reverse();
                for i in buffer {
                    self.stack.push(i as u32);
                }
            }
            _ => {
                println!("function not implemented");
            }
        }
        Ok(())
    }
    fn start_alias(&mut self) -> Option<Box<IError>> {
        for (pos, i) in self.lines.iter().enumerate() {
            if i.contains(':') {
                let mut string = i.to_owned();

                string = Self::remove_comments(&string).to_owned();

                //add alias if it is an alias
                if string.trim().chars().last() == Some(':') {
                    let value = string.trim().replace(":", "");
                    if self.names.get(&value).is_some() {
                        return Some(IError::message(format!(
                            "duplicate alias: {}",
                            value
                        )));
                    }
                    if self.debug {
                        println!("{}: {}", value, pos + 1);
                    }
                    self.names.insert(value, pos + 2);
                }
            }
        }
        None
    }
    fn get_real_value<T>(
        &self,
        mut parameter: &str,
    ) -> Result<T, Box<dyn Error>>
    where
        T: 'static,
        T: FromStr,
        T: From<u32>,
        <T as FromStr>::Err: std::error::Error,
    {
        parameter = Self::remove_comments(parameter);

        let splited: Vec<&str> = parameter.split(' ').collect();

        if splited.len() < 2 {
            return Err(IError::message(format!(
                "malformated param at {}",
                self.pos
            )));
        }
        match splited[0] {
            "comeu" => {
                let mut parse = splited[1].parse::<i32>()?;
                if parse < 0 {
                    parse = -parse;
                    parse = self.pos as i32 - parse;
                }
                if splited[1].contains('+') {
                    parse = self.pos as i32 + parse;
                }
                let a: T = (parse as u32).into();
                Ok(a)
            }
            "chupou" => {
                let value = splited[1].parse::<usize>()?;
                let position = 1 + value;
                if self.stack.len() < position {
                    return Err(IError::message("out of the stack"));
                }
                let a = self.stack[self.stack.len() - position];
                Ok(a.into())
            }
            "fudeu" => {
                let value = splited[1].parse::<usize>()?;
                Ok((self.memory[value] as u32).into())
            }
            "lambeu" => {
                let value = splited[1].trim();
                if value.chars().collect::<Vec<char>>()[0] != ':' {
                    return Err(IError::message(format!(
                        "malformated name in command at {}, '{}'",
                        self.pos, value
                    )));
                }
                let value = value.replace(':', "");

                let a = self.names.get(&value).ok_or(IError::message(
                    format!("cant find {}", value),
                ))?;

                Ok((*a as u32).into())
            }
            token => {
                return Err(IError::message(format!(
                    "unexpected token in command of line {}, '{}'",
                    self.pos, token
                )))
            }
        }
    }

    fn verify_index_overflow(&self, pos: usize) -> bool {
        self.lines.len() <= pos
    }
}

fn run(
    file_path: String,
    debug: bool,
    lines: usize,
) -> Result<(), Box<dyn Error>> {
    if file_path.is_empty() {
        return Err(IError::message(
            "file was not specified, please specify an .robson file",
        ));
    }
    if !file_path.contains(".robson") {
        return Err(IError::message("please specify a .robson file"));
    }
    let mut interpreter = Interpreter::new(&file_path)?;
    interpreter.debug(debug);
    match interpreter.start_alias() {
        Some(err) => return Err(err),
        None => {}
    };
    interpreter.start()?;

    if interpreter.debug {
        print!("[");
        for i in 0..lines {
            print!("{}, ", interpreter.memory[i]);
        }
        print!("]\n");
    }
    Ok(())
}

fn main() {
    let args = &std::env::args().collect::<Vec<String>>();
    let mut file_path = String::new();
    let mut debug = false;
    let mut lines = 0;
    for (i, string) in args.iter().enumerate() {
        match i {
            1 => {
                if string == "--version" {
                    println!("Robson v{}", VERSION);
                    return;
                }
                file_path = string.to_owned();
            }
            2 => {
                if string.to_lowercase() != "debug" {
                    println!("\x1b[93m!invalid flag, flags are: !\x1b[0m");
                    println!("debug");
                } else {
                    debug = true
                }
            }
            3 => {
                if !debug {
                    println!("\x1b[93m!invalid argument!\x1b[0m");
                    println!("{}", string);
                } else {
                    lines = match string.parse::<usize>() {
                        Ok(a) => a,
                        Err(_) => {
                            println!(
                                "\x1b[93m!couldnt parse {} into integer!\x1b[0m",
                                string
                            );
                            0
                        }
                    };
                }
            }
            _ => {}
        }
    }
    if let Err(err) = run(file_path, debug, lines) {
        println!(
            "\x1b[91m\n--------------------\n{:?}\n--------------------\x1b[0m",
            err
        )
    }
}
