use data_struct::IError;

use crate::{
  data_struct::{Type, TypedByte},
  utils::convert_kind_byte,
};

pub mod compiler;
pub mod data_struct;
pub mod interpreter;
#[cfg(test)]
mod tests;
mod utils;

pub trait Infra {
  fn read_line(&mut self) -> Result<String, std::io::Error>;
  fn print(&mut self, to_print: String);
  fn println(&mut self, to_print: String);
  fn flush(&mut self);
  fn enable_raw_mode(&self) -> Result<(), IError>;
  fn disable_raw_mode(&self) -> Result<(), IError>;
  fn clear_purge(&mut self) -> Result<(), IError>;
  fn clear_all(&mut self) -> Result<(), IError>;
  fn poll(&self, duration: u64) -> Result<u32, IError>;
  fn hide_cursor(&mut self) -> Result<(), IError>;
  fn show_cursor(&mut self) -> Result<(), IError>;
  fn move_cursor(&mut self, x: u32, y: u32) -> Result<(), IError>;
}

pub fn print_file_buffer(buffer: Vec<u8>) {
  let mut command = 1;
  let mut index = 0;
  while index < buffer.len() {
    let opcode = buffer[index];
    let kind_byte = buffer[index + 1];
    let param1_byte = [
      buffer[index + 2],
      buffer[index + 3],
      buffer[index + 4],
      buffer[index + 5],
    ];
    let param2_byte = [
      buffer[index + 6],
      buffer[index + 7],
      buffer[index + 8],
      buffer[index + 9],
    ];
    let param3_byte = [
      buffer[index + 10],
      buffer[index + 11],
      buffer[index + 12],
      buffer[index + 13],
    ];

    let types = buffer[index + 14];
    let converted_types = convert_kind_byte(types);
    let param1 = TypedByte {
      value: param1_byte,
      r#type: Type::from(converted_types[0]),
    };
    let param2 = TypedByte {
      value: param2_byte,
      r#type: Type::from(converted_types[1]),
    };
    let param3 = TypedByte {
      value: param3_byte,
      r#type: Type::from(converted_types[2]),
    };

    println!("command: {command}\nopcode: {opcode}\ntypes_byte: {types}\nkind_byte: {kind_byte:08b}\nparam1: {param1:?}\nparam2: {param2:?}\nparam3: {param3:?}\n\n");

    index += 15;
    command += 1;
  }
}
