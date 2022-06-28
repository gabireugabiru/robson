use crate::{infra::Infra, Interpreter};

#[test]
fn push_and_print() {
  let mut interpreter =
    Interpreter::new("tests/push.robson", 200, Infra::new(None))
      .unwrap();
  interpreter.start().unwrap();
}
#[test]
fn jump() {
  let mut interpreter =
    Interpreter::new("tests/jump.robson", 200, Infra::new(None))
      .unwrap();
  assert_eq!(interpreter.start_alias().is_none(), true);
  interpreter.start().unwrap()
}

#[test]
fn memory() {
  let mut interpreter =
    Interpreter::new("tests/memory.robson", 200, Infra::new(None))
      .unwrap();
  interpreter.start().unwrap()
}

#[test]
fn if_() {
  let mut interpreter =
    Interpreter::new("tests/memory.robson", 200, Infra::new(None))
      .unwrap();
  assert_eq!(interpreter.start_alias().is_none(), true);
  interpreter.start().unwrap()
}
#[test]
fn input() {
  let mut interpreter = Interpreter::new(
    "tests/input.robson",
    200,
    Infra::new(Some("teste12321312".to_owned())),
  )
  .unwrap();
  interpreter.start().unwrap()
}
#[test]
fn operations() {
  let mut interpreter = Interpreter::new(
    "tests/operations.robson",
    200,
    Infra::new(None),
  )
  .unwrap();
  assert_eq!(interpreter.start_alias().is_none(), true);
  interpreter.start().unwrap();
}

#[test]
fn types() {
  let mut interpreter =
    Interpreter::new("tests/types.robson", 200, Infra::new(None))
      .unwrap();
  assert_eq!(interpreter.start_alias().is_none(), true);
  interpreter.start().unwrap();
}
