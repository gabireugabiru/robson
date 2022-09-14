use std::{
  io::Write,
  time::{Duration, Instant},
};

use rand::Rng;

use crate::{
  data_struct::{IError, Stack, Type, TypedByte},
  utils::{
    convert_kind_byte, convert_two_bits, f32_mod, i32_mod, u32_mod,
  },
  Infra,
};

use super::utils::{
  approx_equal, f32_add, f32_div, f32_mul, f32_sub, i32_add, i32_div,
  i32_mul, i32_sub, u32_add, u32_div, u32_mul, u32_sub,
};

pub struct Interpreter<'a> {
  pub memory: Vec<TypedByte>,
  pub debug: bool,
  time: Option<Instant>,
  duration: Option<Duration>,
  stack: Stack,
  infra: Box<dyn Infra>,
  index: usize,
  current_command: usize,
  buffer: &'a [u8],
  opcode_functions: [fn(
    &mut Interpreter,
    params: [(TypedByte, usize, bool); 3],
  ) -> Result<(), IError>; 17],
  operations: [[fn(&mut Interpreter, (TypedByte, TypedByte)); 3]; 5],
  convertion_array:
    [fn(TypedByte, &mut Interpreter) -> Result<TypedByte, IError>; 4],
  #[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
  used_input: i64,
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
  if value != 0 {
    return Err(IError::message("chupou is not 0"));
  }
  let top = interpreter.stack.top()?;
  interpreter.stack.pop();
  Ok(top)
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
  let address2 = interpreter.memory[address]
    .force_u32(interpreter.current_command)?
    as usize;
  interpreter.validate_until(address2);
  Ok(interpreter.memory[address2])
}
fn do_no_shit(
  _: &mut Interpreter,
  _: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  Ok(())
}
// OPCODE 1
#[inline(always)]
fn operations(
  interpreter: &mut Interpreter,
  [param1, param2, param3]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let kind = interpreter
    .convert(param1.0, param1.1)?
    .force_u32(interpreter.current_command)?;
  let value = interpreter.convert(param2.0, param2.1)?;
  let value2 = interpreter.convert(param3.0, param3.1)?;
  if param2.2 {
    interpreter.operations[kind as usize][value2.r#type as usize](
      interpreter,
      (value.convert(value2.r#type), value2),
    );
    return Ok(());
  }
  if param3.2 {
    interpreter.operations[kind as usize][value.r#type as usize](
      interpreter,
      (value, value2.convert(value.r#type)),
    );
    return Ok(());
  }
  if value.r#type != value2.r#type {
    return Err(IError::message(format!(
      "Adding with incompatible types at command {}",
      interpreter.current_command
    )));
  }
  interpreter.operations[kind as usize][value.r#type as usize](
    interpreter,
    (value, value2),
  );
  Ok(())
}

//OPCODE 2
#[inline(always)]
fn if_lower(
  interpreter: &mut Interpreter,
  [param1, param2, param3]: [(TypedByte, usize, bool); 3],
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

//OPCODE 3
fn push_stack(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let value = interpreter.convert(param1.0, param1.1)?;
  interpreter.stack.push(value);
  Ok(())
}

//OPCODE 4
fn if_true_jump(
  interpreter: &mut Interpreter,
  [param1, param2, param3]: [(TypedByte, usize, bool); 3],
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

//OPCODE 5
#[inline(always)]
fn vstack_jump(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let value = interpreter
    .convert(param1.0, param1.1)?
    .force_u32(interpreter.current_command)?;
  if interpreter.stack.is_empty() {
    interpreter.index = (value * 15) as usize;
  }
  Ok(())
}

//OPCODE 6
#[inline(always)]
fn input(
  interpreter: &mut Interpreter,
  [param1, param2, param3]: [(TypedByte, usize, bool); 3],
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
      interpreter.validate_until(value);
      interpreter.memory[value] = buff.trim().parse::<u32>()?.into()
    }
    2 => {
      interpreter.validate_until(value);
      interpreter.memory[value] = buff.trim().parse::<i32>()?.into()
    }
    3 => {
      interpreter.validate_until(value);
      interpreter.memory[value] = buff.trim().parse::<f32>()?.into()
    }
    _ => {
      let address_to = value + limit + 2;
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

//OPCODE 7
#[inline(always)]
fn print(
  interpreter: &mut Interpreter,
  [..]: [(TypedByte, usize, bool); 3],
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

//OPCODE 8
#[inline(always)]
fn printnumber(
  interpreter: &mut Interpreter,
  [..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
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

//OPCODE 9
#[inline(always)]
fn jump(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let value = interpreter
    .convert(param1.0, param1.1)?
    .force_u32(interpreter.current_command)?;
  interpreter.index = (value * 15) as usize;
  Ok(())
}

//OPCODE 10
#[inline(always)]
fn set(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize, bool); 3],
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

//OPCODE 11
#[inline(always)]
fn pop_stack(
  interpreter: &mut Interpreter,
  [..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  if !interpreter.stack.is_empty() {
    interpreter.stack.pop();
  }
  Ok(())
}

//OPCODE 12
#[inline(always)]
fn load_string(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let mut value = interpreter
    .convert(param1.0, param1.1)?
    .force_u32(interpreter.current_command)?
    as usize;
  let mut buffer: Vec<u32> = Vec::new();
  interpreter.validate_until(value + 5);
  loop {
    if value == interpreter.memory.len() {
      break;
    }
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

//OPCODE 13
#[inline(always)]
fn time_operations(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  match interpreter
    .convert(param1.0, param1.1)?
    .force_u32(interpreter.current_command)?
  {
    // SET ax
    0 => {
      interpreter.time = Some(Instant::now());
    }
    //SET bx
    1 => {
      let a = interpreter.stack.top()?.value;
      interpreter.stack.pop();
      let b = interpreter.stack.top()?.value;
      let result = [a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3]];

      interpreter.duration =
        Some(Duration::from_millis(u64::from_be_bytes(result)))
    }
    // CMP ax elapsed to bx
    2 => {
      if let Some(a) = interpreter.time {
        if let Some(b) = interpreter.duration {
          let elapsed = a.elapsed();
          if elapsed < b {
            interpreter.stack.push(0u32.into());
          } else if elapsed == b {
            interpreter.stack.push(1u32.into());
          } else {
            interpreter.stack.push(2u32.into());
          }
        }
      }
    }
    _ => {}
  }
  Ok(())
}
//OPCODE 14
#[inline(always)]
fn flush(
  interpreter: &mut Interpreter,
  _: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  interpreter.infra.flush();
  Ok(())
}
//OPCODE 15
#[inline(always)]
fn terminal_commands(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  match interpreter
    .convert(param1.0, param1.1)?
    .force_u32(interpreter.current_command)?
  {
    // RAW MODE
    0 => {
      let on_off = interpreter
        .stack
        .top()?
        .force_u32(interpreter.current_command)?;
      interpreter.stack.pop();
      if on_off == 0 {
        interpreter.infra.disable_raw_mode()?;
      } else {
        interpreter.infra.enable_raw_mode()?;
      }
    }
    // CLEAR
    1 => {
      let r#type = interpreter
        .stack
        .top()?
        .force_u32(interpreter.current_command)?;
      interpreter.stack.pop();
      if r#type == 0 {
        interpreter.infra.clear_purge()?;
      } else {
        interpreter.infra.clear_all()?;
      }
    }
    // POLL KEYBOARD
    2 => {
      let a = interpreter.stack.top()?.value;
      interpreter.stack.pop();
      let b = interpreter.stack.top()?.value;
      interpreter.stack.pop();
      let result = [a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3]];
      let value =
        interpreter.infra.poll(u64::from_be_bytes(result))?;
      interpreter.stack.push(value.into());
    }
    // SHOW/HIDE CURSOR
    3 => {
      let on_off = interpreter
        .stack
        .top()?
        .force_u32(interpreter.current_command)?;
      interpreter.stack.pop();
      if on_off == 0 {
        interpreter.infra.hide_cursor()?;
      } else {
        interpreter.infra.show_cursor()?;
      }
    }
    // MOVE CURSOR
    4 => {
      let x = interpreter
        .stack
        .top()?
        .force_u32(interpreter.current_command)?;
      interpreter.stack.pop();
      let y = interpreter
        .stack
        .top()?
        .force_u32(interpreter.current_command)?;
      interpreter.stack.pop();
      interpreter.infra.move_cursor(x, y)?;
    }
    _ => {}
  }
  Ok(())
}
fn random(
  interpreter: &mut Interpreter,
  _: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let mut rng = rand::thread_rng();
  interpreter.stack.push(rng.gen::<f32>().into());
  Ok(())
}

impl<'a> Interpreter<'a> {
  pub fn new(
    buffer: &'a [u8],
    infra: Box<dyn Infra>,
  ) -> Result<Self, IError> {
    Ok(Self {
      memory: Vec::new(),
      stack: Stack::default(),
      debug: false,
      time: None,
      duration: None,
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
        time_operations,
        flush,
        terminal_commands,
        random,
      ],
      convertion_array: [
        not_convert,
        convert_chupou,
        conver_fudeu,
        convert_penetrou,
      ],
      operations: [
        [
          |int, (v1, v2)| int.stack.push(u32_add(*v1, *v2).into()),
          |int, (v1, v2)| int.stack.push(i32_add(*v1, *v2).into()),
          |int, (v1, v2)| int.stack.push(f32_add(*v1, *v2).into()),
        ],
        [
          |int, (v1, v2)| int.stack.push(u32_sub(*v1, *v2).into()),
          |int, (v1, v2)| int.stack.push(i32_sub(*v1, *v2).into()),
          |int, (v1, v2)| int.stack.push(f32_sub(*v1, *v2).into()),
        ],
        [
          |int, (v1, v2)| int.stack.push(u32_mul(*v1, *v2).into()),
          |int, (v1, v2)| int.stack.push(i32_mul(*v1, *v2).into()),
          |int, (v1, v2)| int.stack.push(f32_mul(*v1, *v2).into()),
        ],
        [
          |int, (v1, v2)| int.stack.push(u32_div(*v1, *v2).into()),
          |int, (v1, v2)| int.stack.push(i32_div(*v1, *v2).into()),
          |int, (v1, v2)| int.stack.push(f32_div(*v1, *v2).into()),
        ],
        [
          |int, (v1, v2)| int.stack.push(u32_mod(*v1, *v2).into()),
          |int, (v1, v2)| int.stack.push(i32_mod(*v1, *v2).into()),
          |int, (v1, v2)| int.stack.push(f32_mod(*v1, *v2).into()),
        ],
      ],
      #[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
      used_input: -1,
    })
  }
  pub fn debug(&mut self, new: bool) {
    self.debug = new;
  }

  pub fn run_buffer(&mut self) -> Result<(), IError> {
    loop {
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

      let [param1_convert, param2_convert] =
        convert_two_bits(converted_kind[3] as u8);
      let [param3_convert, _] =
        convert_two_bits(converted_types[3] as u8);
      if self.debug {
        let command = self.current_command;
        println!("\ncommand: {command}\nopcode: {opcode}\ntypes_byte: {types:08b}\nkind_byte: {kind_byte:08b}\nparam1: {param1}\nparam2: {param2}\nparam3: {param3}\nstack: {}", self.stack);
      }

      self.index += 15;
      self.execute_command(
        opcode,
        (param1, converted_kind[0], param1_convert),
        (param2, converted_kind[1], param2_convert),
        (param3, converted_kind[2], param3_convert),
      )?;
      self.current_command = self.index / 15;
      if self.index >= self.buffer.len() {
        break;
      }
    }
    Ok(())
  }
  pub fn validate_until(&mut self, address: usize) {
    if self.memory.len() <= address {
      let byte: TypedByte = 0u32.into();
      self.memory.resize(address + 1, byte);
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
    param1: (TypedByte, usize, bool),
    param2: (TypedByte, usize, bool),
    param3: (TypedByte, usize, bool),
  ) -> Result<(), IError> {
    self.opcode_functions[opcode as usize](
      self,
      [param1, param2, param3],
    )?;
    Ok(())
  }
}
