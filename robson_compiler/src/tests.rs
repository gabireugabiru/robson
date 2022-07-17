use crate::{compiler::Compiler, interpreter::Interpreter, Infra};

pub struct TestInfra {
  stdin: String,
  stdout: String,
}
impl TestInfra {
  fn new(stdin: String) -> Box<Self> {
    Box::new(Self {
      stdin,
      stdout: String::new(),
    })
  }
}
impl Infra for TestInfra {
  fn print(&mut self, to_print: String) {
    self.stdout.push_str(&to_print);
  }
  fn println(&mut self, to_print: String) {
    self.stdout.push_str(&format!("{}\n", to_print))
  }
  fn read_line(&self) -> Result<String, std::io::Error> {
    Ok(self.stdin.clone())
  }
}

#[test]
fn push_and_print() {
  let mut compiler = Compiler::new(
    "tests/push.robson".to_owned(),
    TestInfra::new("".to_owned()),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();

  let mut interpreter =
    Interpreter::new(&compiled, TestInfra::new(String::new()))
      .unwrap();
  while !interpreter.run_buffer().unwrap() {}
}
#[test]
fn jump() {
  let mut compiler = Compiler::new(
    "tests/jump.robson".to_owned(),
    TestInfra::new("".to_owned()),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();

  let mut interpreter =
    Interpreter::new(&compiled, TestInfra::new(String::new()))
      .unwrap();
  while !interpreter.run_buffer().unwrap() {}
}

#[test]
fn memory() {
  let mut compiler = Compiler::new(
    "tests/memory.robson".to_owned(),
    TestInfra::new("".to_owned()),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();
  let mut interpreter =
    Interpreter::new(&compiled, TestInfra::new(String::new()))
      .unwrap();
  while !interpreter.run_buffer().unwrap() {}
}

#[test]
fn if_() {
  let mut compiler = Compiler::new(
    "tests/if.robson".to_owned(),
    TestInfra::new("".to_owned()),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();

  let mut interpreter =
    Interpreter::new(&compiled, TestInfra::new(String::new()))
      .unwrap();
  while !interpreter.run_buffer().unwrap() {}
}
#[test]
fn input() {
  let mut compiler = Compiler::new(
    "tests/input.robson".to_owned(),
    TestInfra::new("".to_owned()),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();
  let mut interpreter = Interpreter::new(
    &compiled,
    TestInfra::new("testeteste123".to_owned()),
  )
  .unwrap();
  while !interpreter.run_buffer().unwrap() {}
}
#[test]
fn operations() {
  let mut compiler = Compiler::new(
    "tests/operations.robson".to_owned(),
    TestInfra::new("".to_owned()),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();

  let mut interpreter =
    Interpreter::new(&compiled, TestInfra::new(String::new()))
      .unwrap();
  while !interpreter.run_buffer().unwrap() {}
}

#[test]
fn types() {
  let mut compiler = Compiler::new(
    "tests/types.robson".to_owned(),
    TestInfra::new("".to_owned()),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();
  let mut interpreter =
    Interpreter::new(&compiled, TestInfra::new(String::new()))
      .unwrap();
  while !interpreter.run_buffer().unwrap() {}
}
