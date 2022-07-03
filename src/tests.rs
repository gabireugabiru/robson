use crate::{infra::Infra, Interpreter};

#[test]
fn push_and_print() {
  let mut interpreter = Interpreter::new(
    "tests/push.robson",
    200,
    Infra::new(String::new()),
  )
  .unwrap();
  while interpreter.execute_line().unwrap().is_none() {}
}
#[test]
fn jump() {
  let mut interpreter = Interpreter::new(
    "tests/jump.robson",
    200,
    Infra::new(String::new()),
  )
  .unwrap();
  assert_eq!(interpreter.start_alias().is_none(), true);
  while interpreter.execute_line().unwrap().is_none() {}
}

#[test]
fn memory() {
  let mut interpreter = Interpreter::new(
    "tests/memory.robson",
    200,
    Infra::new(String::new()),
  )
  .unwrap();
  while interpreter.execute_line().unwrap().is_none() {}
}

#[test]
fn if_() {
  let mut interpreter = Interpreter::new(
    "tests/if.robson",
    200,
    Infra::new(String::new()),
  )
  .unwrap();
  assert_eq!(interpreter.start_alias().is_none(), true);
  while interpreter.execute_line().unwrap().is_none() {}
}
#[test]
fn input() {
  let mut interpreter = Interpreter::new(
    "tests/input.robson",
    200,
    Infra::new("teste12321312".to_owned()),
  )
  .unwrap();
  while interpreter.execute_line().unwrap().is_none() {}
}
#[test]
fn operations() {
  let mut interpreter = Interpreter::new(
    "tests/operations.robson",
    200,
    Infra::new(String::new()),
  )
  .unwrap();
  assert_eq!(interpreter.start_alias().is_none(), true);
  while interpreter.execute_line().unwrap().is_none() {}
}

#[test]
fn types() {
  let mut interpreter = Interpreter::new(
    "tests/types.robson",
    200,
    Infra::new(String::new()),
  )
  .unwrap();
  assert_eq!(interpreter.start_alias().is_none(), true);
  while interpreter.execute_line().unwrap().is_none() {}
}
