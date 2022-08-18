use std::{
  collections::HashMap,
  fs::OpenOptions,
  io::{BufRead, BufReader},
};

use crate::{
  data_struct::{IError, TypedByte},
  utils::{self, create_kind_byte},
  Infra,
};

pub struct Compiler {
  lines: Vec<String>,
  opcode_params: [u8; 16],
  names: HashMap<String, usize>,
  pos: usize,
  debug: bool,
  buffer: Vec<u8>,
  infra: Box<dyn Infra>,
  last_opcode: u8,
}
impl Compiler {
  pub fn new(
    path: String,
    infra: Box<dyn Infra>,
  ) -> Result<Self, IError> {
    let file = OpenOptions::new().read(true).open(path)?;

    let buff_reader = BufReader::new(&file);
    let lines = buff_reader
      .lines()
      .flat_map(|a| a.ok())
      .collect::<Vec<String>>();
    Ok(Self {
      buffer: Vec::new(),
      debug: false,
      infra,
      last_opcode: 0,
      lines,
      names: HashMap::new(),
      opcode_params: [0, 3, 3, 1, 3, 1, 3, 0, 0, 1, 1, 0, 1, 1, 0, 1],
      pos: 0,
    })
  }
  pub fn compile(&mut self) -> Result<Vec<u8>, IError> {
    self.start_command_alias();
    loop {
      if self.verify_index_overflow(self.pos) {
        break;
      }
      let pre_string = self.lines[self.pos].to_owned();
      let mut string = pre_string.as_str();

      string = Self::remove_comments(string).trim();

      // skip aliases
      if string.ends_with(':') {
        self.pos += 1;
        continue;
      }

      // //skip spaces
      if string.is_empty() {
        self.pos += 1;
        continue;
      }

      // Implements the push abreviation
      if self.last_opcode == 3 && !string.contains("robson") {
        self.push_command(
          3,
          [string.to_owned(), "".to_owned(), "".to_owned()],
        )?;
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
      if opcode as usize >= self.opcode_params.len() {
        return Err(IError::message(format!(
          "invalid opcode of line {}",
          self.pos + 1
        )));
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

      self.push_command(opcode, params)?;

      self.last_opcode = opcode;
    }
    Ok(self.buffer.clone())
  }
  pub fn start_command_alias(&mut self) -> Option<IError> {
    struct LastCommand {
      value: u8,
      pos: usize,
    }
    let mut command_number = 0;
    let mut last_command = LastCommand { pos: 0, value: 0 };
    for (pos, i) in self.lines.iter().enumerate() {
      let string = Self::remove_comments(i).trim().to_owned();
      if string.is_empty() {
        continue;
      }
      if string.contains(':') {
        //add alias if it is an alias
        if string.ends_with(':') {
          let value = string.trim().replace(':', "");
          if self.names.get(&value).is_some() {
            return Some(IError::message(format!(
              "duplicate alias: {}",
              value
            )));
          }
          if self.debug {
            self.infra.println(format!("{}: {}", value, pos + 1));
          }
          self.names.insert(value, command_number);
        }
      } else {
        //if is not an alias add the command
        if string.contains("robson") {
          command_number += 1;
          let mut opcode: u8 = 0;
          let spaces: Vec<&str> = string.split(' ').collect();

          for i in spaces {
            if i != "robson" {
              return Some(IError::message(format!(
                "invalid token for opcode in line {}, '{}'",
                pos + 1,
                i
              )));
            }
            opcode += 1;
          }
          last_command.value = opcode;
          last_command.pos = pos;
        } else if last_command.value == 3
          && last_command.pos + 1 != pos
        {
          command_number += 1;
        }
      }
    }
    None
  }
  pub fn remove_comments(string: &str) -> &str {
    let mut res = string;

    let comments = string.split(';').collect::<Vec<&str>>();
    if !comments.is_empty() {
      res = comments[0].trim();
    }
    res
  }
  fn verify_index_overflow(&self, pos: usize) -> bool {
    self.lines.len() <= pos
  }
  pub fn push_command(
    &mut self,
    opcode: u8,
    params: [String; 3],
  ) -> Result<(), IError> {
    self.buffer.push(opcode);
    let (param1, param1_kind, param1_types) =
      self.get_kind_value(params[0].trim())?;
    let param1 = param1.value;

    let (param2, param2_kind, param2_types) =
      self.get_kind_value(params[1].trim())?;
    let param2 = param2.value;

    let (param3, param3_kind, param3_types) =
      self.get_kind_value(params[2].trim())?;
    let param3 = param3.value;

    self.buffer.push(utils::create_kind_byte(
      param1_kind,
      param2_kind,
      param3_kind,
      0,
    ));

    for i in param1 {
      self.buffer.push(i);
    }
    for i in param2 {
      self.buffer.push(i);
    }
    for i in param3 {
      self.buffer.push(i);
    }
    self.buffer.push(create_kind_byte(
      param1_types,
      param2_types,
      param3_types,
      0,
    ));
    Ok(())
  }
  pub fn get_kind_value(
    &self,
    parameter: &str,
  ) -> Result<(TypedByte, u8, u8), IError> {
    if parameter.is_empty() {
      return Ok((0u32.into(), 0, 0));
    }
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
            value = value.replace('f', "");
            Ok((value.parse::<f32>()?.into(), 0, 2))
          }
          'i' => {
            value = value.replace('i', "");
            Ok((value.parse::<i32>()?.into(), 0, 1))
          }
          _ => Ok((splited[1].trim().parse::<u32>()?.into(), 0, 0)),
        }
      }
      "chupou" => {
        let value = splited[1].parse::<u32>()?;
        Ok((value.into(), 1, 0))
      }
      "fudeu" => {
        let value = splited[1].trim().parse::<u32>()?;
        Ok((value.into(), 2, 0))
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

        let a = self.names.get(&value).ok_or_else(|| {
          IError::message(format!("cant find {}", value))
        })?;
        Ok(((*a as u32).into(), 0, 0))
      }
      "penetrou" => {
        let value = splited[1].trim().parse::<u32>()?;
        Ok((value.into(), 3, 0))
      }
      token => {
        return Err(IError::message(format!(
          "unexpected token in command of line {}, '{}'",
          self.pos, token
        )))
      }
    }
  }
}
