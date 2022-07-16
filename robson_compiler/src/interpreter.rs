use std::io::Write;

use crate::{
  data_struct::{IError, Stack, Type, TypedByte},
  utils::convert_kind_byte,
  Infra,
};

use super::utils::{
  approx_equal, f32_add, f32_div, f32_mul, f32_sub, i32_add, i32_div,
  i32_mul, i32_sub, u32_add, u32_div, u32_mul, u32_sub,
};

pub struct Interpreter<'a> {
  pub memory: Vec<TypedByte>,
  stack: Stack,
  pub debug: bool,
  infra: Box<dyn Infra>,
  index: usize,
  current_command: usize,
  buffer: &'a [u8],
  #[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
  used_input: i64,
  opcode_functions: [fn(
    &mut Interpreter,
    params: [(TypedByte, usize); 3],
  ) -> Result<(), IError>; 13],
  convertion_array:
    [fn(TypedByte, &mut Interpreter) -> Result<TypedByte, IError>; 4],
}
#[inline(always)]
fn not_convert(
  byte: TypedByte,
  _interpreter: &mut Interpreter,
) -> Result<TypedByte, IError> {
  Ok(byte)
}

#[inline(always)]
fn convert_chupou(
  byte: TypedByte,
  interpreter: &mut Interpreter,
) -> Result<TypedByte, IError> {
  let value = u32::from_be_bytes(*byte);
  let position = (1 + value) as usize;
  if interpreter.stack.len() < position {
    Err(IError::message("out of the stack"))
  } else {
    Ok(interpreter.stack[interpreter.stack.len() - position])
  }
}

#[inline(always)]
fn conver_fudeu(
  byte: TypedByte,
  interpreter: &mut Interpreter,
) -> Result<TypedByte, IError> {
  let address = byte.force_u32(interpreter.current_command)? as usize;
  interpreter.validate_until(address);

  Ok(interpreter.memory[address])
}

#[inline(always)]
fn convert_penetrou(
  byte: TypedByte,
  interpreter: &mut Interpreter,
) -> Result<TypedByte, IError> {
  let address = byte.force_u32(interpreter.current_command)? as usize;
  interpreter.validate_until(address);

  Ok(interpreter.memory[address])
}
fn do_no_shit(
  _: &mut Interpreter,
  _: [(TypedByte, usize); 3],
) -> Result<(), IError> {
  Ok(())
}

#[inline(always)]
fn operations(
  interpreter: &mut Interpreter,
  [param1, param2, param3]: [(TypedByte, usize); 3],
) -> Result<(), IError> {
  let kind = interpreter
    .convert(param1.0, param1.1)?
    .force_u32(interpreter.current_command)?;
  let value = interpreter.convert(param2.0, param2.1)?;
  let value2 = interpreter.convert(param3.0, param3.1)?;
  if value.r#type != value2.r#type {
    return Err(IError::message(format!(
      "Adding with incompatible types at command {}",
      interpreter.current_command
    )));
  }
  match kind {
    0 => match value.r#type {
      Type::Usigned => {
        interpreter.stack.push(u32_add(*value, *value2).into())
      }
      Type::Signed => {
        interpreter.stack.push(i32_add(*value, *value2).into())
      }
      Type::Floating => {
        interpreter.stack.push(f32_add(*value, *value2).into())
      }
    },
    1 => match value.r#type {
      Type::Signed => {
        interpreter.stack.push(i32_sub(*value, *value2).into())
      }
      Type::Usigned => {
        interpreter.stack.push(u32_sub(*value, *value2).into())
      }
      Type::Floating => {
        interpreter.stack.push(f32_sub(*value, *value2).into())
      }
    },
    2 => match value.r#type {
      Type::Signed => {
        interpreter.stack.push(i32_mul(*value, *value2).into())
      }
      Type::Usigned => {
        interpreter.stack.push(u32_mul(*value, *value2).into())
      }
      Type::Floating => {
        interpreter.stack.push(f32_mul(*value, *value2).into())
      }
    },
    3 => match value.r#type {
      Type::Signed => {
        interpreter.stack.push(i32_div(*value, *value2).into())
      }
      Type::Usigned => {
        interpreter.stack.push(u32_div(*value, *value2).into())
      }
      Type::Floating => {
        interpreter.stack.push(f32_div(*value, *value2).into())
      }
    },
    _ => {
      return Err(IError::message("This function is not implemented"))
    }
  }
  Ok(())
}

#[inline(always)]
fn if_lower(
  interpreter: &mut Interpreter,
  [param1, param2, param3]: [(TypedByte, usize); 3],
) -> Result<(), IError> {
  let value = interpreter.convert(param1.0, param1.1)?;
  let value2 = interpreter.convert(param2.0, param2.1)?;
  let pos = interpreter
    .convert(param3.0, param3.1)?
    .force_u32(interpreter.current_command)?;
  if value.r#type != value2.r#type {
    return Err(IError::message(format!(
      "Comparing with incompatible types {}",
      interpreter.current_command
    )));
  }
  if *value < *value2 {
    interpreter.index = (pos * 15) as usize;
  }
  Ok(())
}

fn push_stack(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize); 3],
) -> Result<(), IError> {
  let value = interpreter.convert(param1.0, param1.1)?;
  interpreter.stack.push(value);
  Ok(())
}
fn if_true_jump(
  interpreter: &mut Interpreter,
  [param1, param2, param3]: [(TypedByte, usize); 3],
) -> Result<(), IError> {
  let value = interpreter.convert(param1.0, param1.1)?;
  let value2 = interpreter.convert(param2.0, param2.1)?;
  let pos = interpreter
    .convert(param3.0, param3.1)?
    .force_u32(interpreter.current_command)?;

  if value.r#type != value2.r#type {
    return Err(IError::message("Comparing incompatible types"));
  }

  if value.r#type == Type::Floating {
    let value = f32::from_be_bytes(*value);
    let value2 = f32::from_be_bytes(*value2);
    if approx_equal(value, value2, 4) {
      interpreter.index = (pos * 15) as usize;
    }
  } else if *value == *value2 {
    interpreter.index = (pos * 15) as usize;
  };
  Ok(())
}

#[inline(always)]
fn vstack_jump(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize); 3],
) -> Result<(), IError> {
  let value = interpreter
    .convert(param1.0, param1.1)?
    .force_u32(interpreter.current_command)?;
  if interpreter.stack.is_empty() {
    interpreter.index = (value * 15) as usize;
  }
  Ok(())
}

#[inline(always)]
fn input(
  interpreter: &mut Interpreter,
  [param1, param2, param3]: [(TypedByte, usize); 3],
) -> Result<(), IError> {
  let mut value = interpreter
    .convert(param1.0, param1.1)?
    .force_u32(interpreter.current_command)?
    as usize;
  let kind = interpreter
    .convert(param2.0, param2.1)?
    .force_u32(interpreter.current_command)?;
  let limit = interpreter
    .convert(param3.0, param3.1)?
    .force_u32(interpreter.current_command)? as usize;

  std::io::stdout().flush()?;
  let buff = interpreter.infra.read_line()?;

  match kind {
    1 => {
      interpreter.memory[value] = buff.trim().parse::<u32>()?.into()
    }
    2 => {
      interpreter.memory[value] = buff.trim().parse::<i32>()?.into()
    }
    3 => {
      interpreter.memory[value] = buff.trim().parse::<f32>()?.into()
    }
    _ => {
      let address_to = value + limit;
      interpreter.validate_until(address_to as usize);
      for (i, char) in buff.chars().enumerate() {
        if i < limit as usize {
          let char = if char == '\n' { '\0' } else { char };
          interpreter.memory[value] = (char as u32).into();
          value += 1;
        } else {
          break;
        }
      }
      interpreter.memory[value] = 0u32.into();
    }
  };

  Ok(())
}

#[inline(always)]
fn print(
  interpreter: &mut Interpreter,
  [..]: [(TypedByte, usize); 3],
) -> Result<(), IError> {
  if interpreter.stack.is_empty() {
    return Err(IError::message(format!(
      "Trying to use the stack while empty in command {}",
      interpreter.current_command
    )));
  }
  let stack_byte = interpreter.stack.top()?;
  if stack_byte.r#type != Type::Usigned {
    return Err(IError::message("Invalid number type for ASCII"));
  }
  interpreter.infra.print(format!(
    "{}",
    (String::from_utf8_lossy(&[
      u32::from_be_bytes(*stack_byte) as u8
    ]))
  ));

  interpreter.stack.pop();
  Ok(())
}

#[inline(always)]
fn printnumber(
  interpreter: &mut Interpreter,
  [..]: [(TypedByte, usize); 3],
) -> Result<(), IError> {
  if interpreter.stack.is_empty() {
    return Err(IError::message(format!(
      "trying to use the stack while empty at command {}",
      interpreter.current_command
    )));
  }
  let TypedByte { value, r#type } = interpreter.stack.top()?;

  match r#type {
    Type::Floating => interpreter
      .infra
      .print(format!("{}", f32::from_be_bytes(value))),
    Type::Signed => interpreter
      .infra
      .print(format!("{}", i32::from_be_bytes(value))),
    Type::Usigned => interpreter
      .infra
      .print(format!("{}", u32::from_be_bytes(value))),
  }

  interpreter.stack.pop();
  Ok(())
}

#[inline(always)]
fn jump(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize); 3],
) -> Result<(), IError> {
  let value = interpreter
    .convert(param1.0, param1.1)?
    .force_u32(interpreter.current_command)?;
  interpreter.index = (value * 15) as usize;
  Ok(())
}

#[inline(always)]
fn set(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize); 3],
) -> Result<(), IError> {
  let address = interpreter
    .convert(param1.0, param1.1)?
    .force_u32(interpreter.current_command)? as usize;
  let typed_byte = interpreter.stack.top()?;

  interpreter.stack.pop();
  interpreter.validate_until(address);
  interpreter.memory[address] = typed_byte;
  Ok(())
}

#[inline(always)]
fn pop_stack(
  interpreter: &mut Interpreter,
  [..]: [(TypedByte, usize); 3],
) -> Result<(), IError> {
  if !interpreter.stack.is_empty() {
    interpreter.stack.pop();
  }
  Ok(())
}

#[inline(always)]
fn load_string(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize); 3],
) -> Result<(), IError> {
  let mut value = interpreter
    .convert(param1.0, param1.1)?
    .force_u32(interpreter.current_command)?
    as usize;
  let mut buffer: Vec<u32> = Vec::new();
  interpreter.validate_until(value + 5);
  loop {
    let temp = u32::from_be_bytes(*interpreter.memory[value]);
    if temp != 0 {
      buffer.push(temp);
      value += 1;
    } else {
      break;
    }
  }
  buffer.reverse();
  for i in buffer {
    interpreter.stack.push(TypedByte {
      value: i.to_be_bytes(),
      r#type: Type::Usigned,
    });
  }
  Ok(())
}

impl<'a> Interpreter<'a> {
  pub fn new(
    buffer: &'a [u8],
    infra: Box<dyn Infra>,
  ) -> Result<Self, IError> {
    Ok(Self {
      memory: vec![
        TypedByte {
          value: [0; 4],
          r#type: Type::Usigned
        };
        100
      ],
      stack: Stack::default(),
      debug: false,
      infra,
      index: 0,
      buffer,
      current_command: 0,
      opcode_functions: [
        do_no_shit,
        operations,
        if_lower,
        push_stack,
        if_true_jump,
        vstack_jump,
        input,
        print,
        printnumber,
        jump,
        set,
        pop_stack,
        load_string,
      ],
      convertion_array: [
        not_convert,
        convert_chupou,
        conver_fudeu,
        convert_penetrou,
      ],
      #[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
      used_input: -1,
    })
  }
  pub fn debug(&mut self, new: bool) {
    self.debug = new;
  }

  pub fn run_buffer(&mut self) -> Result<bool, IError> {
    let opcode = self.buffer[self.index];
    let kind_byte = self.buffer[self.index + 1];
    let param1_byte = [
      self.buffer[self.index + 2],
      self.buffer[self.index + 3],
      self.buffer[self.index + 4],
      self.buffer[self.index + 5],
    ];
    let param2_byte = [
      self.buffer[self.index + 6],
      self.buffer[self.index + 7],
      self.buffer[self.index + 8],
      self.buffer[self.index + 9],
    ];
    let param3_byte = [
      self.buffer[self.index + 10],
      self.buffer[self.index + 11],
      self.buffer[self.index + 12],
      self.buffer[self.index + 13],
    ];

    let types = self.buffer[self.index + 14];
    let converted_types = convert_kind_byte(types);
    let converted_kind = convert_kind_byte(kind_byte);

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

    if self.debug {
      let command = self.current_command;
      println!("\ncommand: {command}\nopcode: {opcode}\ntypes_byte: {types}\nkind_byte: {kind_byte:08b}\nparam1: {param1}\nparam2: {param2}\nparam3: {param3}\nstack: {}", self.stack);
    }
    self.index += 15;
    self.execute_command(
      opcode,
      (param1, converted_kind[0]),
      (param2, converted_kind[1]),
      (param3, converted_kind[2]),
    )?;
    self.current_command = self.index / 15;
    if self.index < self.buffer.len() {
      Ok(false)
    } else {
      Ok(true)
    }
  }
  pub fn validate_until(&mut self, address: usize) {
    while self.memory.len() <= address {
      self.memory.push(0u32.into());
    }
  }
  pub fn convert(
    &mut self,
    byte: TypedByte,
    r#type: usize,
  ) -> Result<TypedByte, IError> {
    let a = self.convertion_array[r#type](byte, self);
    a
  }
  #[inline(always)]
  pub fn execute_command(
    &mut self,
    opcode: u8,
    param1: (TypedByte, usize),
    param2: (TypedByte, usize),
    param3: (TypedByte, usize),
  ) -> Result<(), IError> {
    self.opcode_functions[opcode as usize](
      self,
      [param1, param2, param3],
    )?;
    Ok(())
  }
}
