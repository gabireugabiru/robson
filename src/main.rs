#[cfg(test)]
mod infra {
  pub struct Infra {
    value: Option<String>,
  }
  impl Infra {
    pub fn new(value: Option<String>) -> Self {
      Self { value }
    }
    pub fn read_line(&self) -> Result<String, std::io::Error> {
      self
        .value
        .as_ref()
        .ok_or(std::io::Error::new(
          std::io::ErrorKind::InvalidData,
          "invalid data",
        ))
        .cloned()
    }
  }
}
#[cfg(not(test))]
mod infra {
  use std::io::stdin;
  pub struct Infra {
    value: Option<String>,
  }
  impl Infra {
    pub fn new(value: Option<String>) -> Self {
      Self { value }
    }
    pub fn read_line(&self) -> Result<String, std::io::Error> {
      if self.value.is_some() {
        return Err(std::io::Error::new(
          std::io::ErrorKind::InvalidData,
          "invalid data",
        ));
      }
      let mut buff = String::new();
      stdin().read_line(&mut buff)?;
      Ok(buff)
    }
  }
}
#[cfg(test)]
mod tests;
mod utils;
use std::{
  collections::HashMap,
  error::Error,
  fmt::Display,
  fs::OpenOptions,
  io::{stdout, BufRead, BufReader, Write},
  ops::{Deref, DerefMut},
};

use crossterm::{
  execute,
  style::{Color, Print, ResetColor, SetForegroundColor},
};
use infra::Infra;
use utils::{
  approx_equal, f32_add, f32_sub, i32_add, i32_sub, u32_add, u32_sub,
};

const VERSION: &str = "0.0.7";

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
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TypedByte {
  value: [u8; 4],
  r#type: Type,
}
impl From<u32> for TypedByte {
  fn from(value: u32) -> Self {
    Self {
      value: value.to_be_bytes(),
      r#type: Type::Usigned,
    }
  }
}
impl From<i32> for TypedByte {
  fn from(value: i32) -> Self {
    Self {
      value: value.to_be_bytes(),
      r#type: Type::Signed,
    }
  }
}
impl From<f32> for TypedByte {
  fn from(value: f32) -> Self {
    Self {
      value: value.to_be_bytes(),
      r#type: Type::Floating,
    }
  }
}
impl Deref for TypedByte {
  type Target = [u8; 4];
  fn deref(&self) -> &Self::Target {
    &self.value
  }
}
impl TypedByte {
  pub fn force_u32(&self, pos: usize) -> Result<u32, Box<dyn Error>> {
    if self.r#type != Type::Usigned {
      return Err(IError::message(format!(
        "Invalid number type at {}",
        pos
      )));
    } else {
      Ok(u32::from_be_bytes(self.value))
    }
  }
}

#[derive(Default, Debug)]
pub struct Stack {
  vec: Vec<TypedByte>,
}
impl Deref for Stack {
  type Target = Vec<TypedByte>;
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
  pub fn top(&self) -> Result<TypedByte, Box<dyn Error>> {
    if !self.vec.is_empty() {
      Ok(self.vec[self.len() - 1])
    } else {
      Err(IError::message(
        "trying to access the stack while it is empty",
      ))
    }
  }
}
#[derive(Clone, Copy, PartialEq, Debug, Eq, PartialOrd)]
pub enum Type {
  Usigned,
  Signed,
  Floating,
}
impl Default for Type {
  fn default() -> Self {
    Self::Usigned
  }
}
pub struct Interpreter {
  memory: Vec<TypedByte>,
  stack: Stack,
  lines: Vec<String>,
  opcode_params: [u8; 14],
  names: HashMap<String, usize>,
  pos: usize,
  debug: bool,
  infra: Infra,
  last_opcode: u8,
}

impl Interpreter {
  pub fn new(
    path: &str,
    limit: usize,
    infra: Infra,
  ) -> Result<Self, Box<dyn Error>> {
    let file = OpenOptions::new().read(true).open(path)?;

    let buff_reader = BufReader::new(&file);
    let lines = buff_reader
      .lines()
      .flat_map(|a| a.ok())
      .collect::<Vec<String>>();
    Ok(Self {
      memory: vec![
        TypedByte {
          value: [0; 4],
          r#type: Type::Usigned
        };
        limit
      ],
      stack: Stack::default(),
      lines,
      opcode_params: [0, 2, 2, 1, 3, 1, 3, 0, 0, 1, 1, 1, 1, 0],
      pos: 0,
      debug: false,
      names: HashMap::new(),
      infra,
      last_opcode: 0,
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
    // let mut last_opcode = 0;
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

      // //skip spaces
      if string.trim().is_empty() {
        self.pos += 1;
        continue;
      }

      // Implements the push abreviation
      if self.last_opcode == 3
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

      let spaces: Vec<&str> = string.split(' ').collect();

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
        println!("\npos: {}", self.pos);
        println!("opcode: {}", opcode);
        println!("count: {}", param_count);
        println!(
          "params: {}, {}, {}",
          params[0], params[1], params[2]
        );
        println!("string '{}'", string);
        println!("stack {:?}", self.stack.vec);
      }
      self.last_opcode = opcode;
      self.command(
        opcode,
        Self::remove_comments(&params[0]),
        Self::remove_comments(&params[1]),
        Self::remove_comments(&params[2]),
      )?;
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
        let value = self.get_real_value(param1)?;
        let value2 = self.get_real_value(param2)?;
        if value.r#type != value2.r#type {
          return Err(IError::message(format!(
            "Adding with incompatible types {}",
            self.pos
          )));
        }

        match value.r#type {
          Type::Usigned => {
            self.stack.push(u32_add(*value, *value2).into())
          }
          Type::Signed => {
            self.stack.push(i32_add(*value, *value2).into())
          }
          Type::Floating => {
            self.stack.push(f32_add(*value, *value2).into())
          }
        }
      }

      //SUBTRACT TWO VALUES
      2 => {
        let value = self.get_real_value(param1)?;
        let value2 = self.get_real_value(param2)?;

        if value.r#type != value2.r#type {
          return Err(IError::message(format!(
            "Adding with incompatible types {}",
            self.pos
          )));
        }

        match value.r#type {
          Type::Signed => {
            self.stack.push(i32_sub(*value, *value2).into())
          }
          Type::Usigned => {
            self.stack.push(u32_sub(*value, *value2).into())
          }
          Type::Floating => {
            self.stack.push(f32_sub(*value, *value2).into())
          }
        }
      }

      //PUSH TO STACK SOME VALUE
      3 => {
        let value = self.get_real_value(param1)?;
        self.stack.push(value);
      }
      //IF TRUE JUMP
      4 => {
        let value = self.get_real_value(param1)?;
        let value2 = self.get_real_value(param2)?;
        let pos = self.get_real_value(param3)?.force_u32(self.pos)?;

        if value.r#type == Type::Floating {
          let value = f32::from_be_bytes(*value);
          let value2 = f32::from_be_bytes(*value2);
          if approx_equal(value, value2, 4) {
            self.pos = (pos - 1) as usize;
          }
        } else {
          self.pos = (pos - 1) as usize;
        }
      }
      //VERIFY THE STACK IF IS EMPTY JUMP
      5 => {
        let value =
          self.get_real_value(param1)?.force_u32(self.pos)?;
        if self.stack.is_empty() {
          self.pos = (value - 1) as usize;
        }
      }
      //GET INPUT AND SET TO A ADDRESS
      6 => {
        let mut value =
          self.get_real_value(param1)?.force_u32(self.pos)?;
        let kind =
          self.get_real_value(param2)?.force_u32(self.pos)?;
        let limit =
          self.get_real_value(param3)?.force_u32(self.pos)?;

        std::io::stdout().flush()?;
        let buff = self.infra.read_line()?;

        match kind {
          1 => {
            self.memory[value as usize] =
              buff.trim().parse::<u32>()?.into()
          }
          _ => {
            for (i, char) in buff.chars().enumerate() {
              if i < limit as usize {
                let char = if char == '\n' { '\0' } else { char };
                self.memory[value as usize] = (char as u32).into();
                value += 1;
              } else {
                break;
              }
            }
            self.memory[value as usize] = 0u32.into();
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
        let stack_byte = self.stack.top()?;
        if stack_byte.r#type != Type::Usigned {
          return Err(IError::message(
            "Invalid number type for ASCII",
          ));
        }
        print!("{}", (u32::from_be_bytes(*stack_byte) as u8) as char);
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
        let TypedByte { value, r#type } = self.stack.top()?;

        match r#type {
          Type::Floating => print!("{}", f32::from_be_bytes(value)),
          Type::Signed => print!("{}", i32::from_be_bytes(value)),
          Type::Usigned => print!("{}", u32::from_be_bytes(value)),
        }

        self.stack.pop();
      }

      //JUMP
      9 => {
        let value =
          self.get_real_value(param1)?.force_u32(self.pos)?;
        self.pos = (value - 1) as usize;
      }

      //SET TO MEMEORY
      10 => {
        let address =
          self.get_real_value(param1)?.force_u32(self.pos)?;
        let typed_byte = self.stack.top()?;

        self.stack.pop();
        self.memory[address as usize] = typed_byte;
      }
      //POP STACK
      11 => {
        if !self.stack.is_empty() {
          self.stack.pop();
        }
      }

      //GET ALL THE STRING BUFFER
      12 => {
        let mut value =
          self.get_real_value(param1)?.force_u32(self.pos)?;
        let mut buffer: Vec<u32> = Vec::new();

        loop {
          let temp = u32::from_be_bytes(*self.memory[value as usize]);
          if temp != 0 {
            buffer.push(temp);
            value += 1;
          } else {
            break;
          }
        }
        buffer.reverse();
        for i in buffer {
          self.stack.push(TypedByte {
            value: i.to_be_bytes(),
            r#type: Type::Usigned,
          });
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
  fn get_real_value(
    &self,
    parameter: &str,
  ) -> Result<TypedByte, Box<dyn Error>> {
    let splited: Vec<&str> = parameter.split(' ').collect();

    if splited.len() < 2 {
      return Err(IError::message(format!(
        "malformated param at {}",
        self.pos
      )));
    }
    match splited[0] {
      "comeu" => {
        let mut value = splited[1].trim().to_owned();
        let first = value.chars().collect::<Vec<char>>()[0];
        match first {
          'f' => {
            value = value.replace("f", "");
            Ok(value.parse::<f32>()?.into())
          }
          'i' => {
            value = value.replace('i', "");
            Ok(value.parse::<i32>()?.into())
          }
          _ => Ok(splited[1].trim().parse::<u32>()?.into()),
        }
      }
      "chupou" => {
        let value = splited[1].parse::<usize>()?;
        let position = 1 + value;
        if self.stack.len() < position {
          return Err(IError::message("out of the stack"));
        }
        let a = self.stack[self.stack.len() - position];
        Ok(a)
      }
      "fudeu" => {
        let value = splited[1].parse::<usize>()?;
        Ok(self.memory[value])
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

        let a = self
          .names
          .get(&value)
          .ok_or(IError::message(format!("cant find {}", value)))?;

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
  let mut interpreter =
    Interpreter::new(&file_path, 131072, Infra::new(None))?;
  interpreter.debug(debug);
  match interpreter.start_alias() {
    Some(err) => return Err(err),
    None => {}
  };
  interpreter.start()?;

  if interpreter.debug {
    print!("[");
    for i in 0..lines {
      print!("{:?}, ", interpreter.memory[i]);
    }
    print!("]\n");
  }
  Ok(())
}

pub fn color_print<T>(string: T, color: Color)
where
  T: Display,
{
  if execute!(
    stdout(),
    SetForegroundColor(color),
    Print(format!("{}\n", string)),
    ResetColor
  )
  .is_err()
  {
    println!("{}", string);
  }
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
          color_print(
            "!invalid flag, flags are: !\nDebug\n-----------------",
            Color::Yellow,
          );
        } else {
          debug = true
        }
      }
      3 => {
        if !debug {
          color_print(
            format!("!invalid argument {}!", string),
            Color::Yellow,
          );
        } else {
          lines = match string.parse::<usize>() {
            Ok(a) => a,
            Err(_) => {
              color_print(
                format!("!couldnt parse {} into integer!", string),
                Color::Yellow,
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
    color_print(
      format!(
        "\n--------------------\n{:?}\n--------------------",
        err
      ),
      Color::Red,
    );
  }
}
