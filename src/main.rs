mod utils;
use std::{error::Error, io::stdin};

use robson_compiler::{
  interpreter::{IError, Interpreter},
  Infra,
};

pub struct RunInfra {}
impl RunInfra {
  pub fn new() -> Box<Self> {
    Box::new(Self {})
  }
}
impl Infra for RunInfra {
  fn print(&mut self, to_print: String) {
    print!("{}", to_print);
  }
  fn println(&mut self, to_print: String) {
    println!("{}", to_print);
  }
  fn read_line(&self) -> Result<String, std::io::Error> {
    let mut buffer = String::new();
    stdin().read_line(&mut buffer)?;
    Ok(buffer)
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
    Interpreter::new(&file_path, 131072, RunInfra::new())?;

  interpreter.debug(debug);
  match interpreter.start_alias() {
    Some(err) => return Err(err),
    None => {}
  };
  let mut result = Ok::<(), Box<dyn Error>>(());
  loop {
    match interpreter.execute_line() {
      Ok(a) => {
        if a.is_some() {
          break;
        }
      }
      Err(interpreter_err) => {
        result = Err(interpreter_err);
        break;
      }
    }
  }

  if interpreter.debug {
    print!("[");
    for i in 0..lines {
      print!("{:?}, ", interpreter.memory[i]);
    }
    print!("]\n");
  }
  result
}

fn main() {
  use crossterm::style::Color;
  use utils::color_print;

  const VERSION: &str = "0.0.9";

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
  use std::time::Instant;
  let now = Instant::now();
  if let Err(err) = run(file_path, debug, lines) {
    color_print(
      format!(
        "\n--------------------\n{:?}\n--------------------",
        err
      ),
      Color::Red,
    );
  }
  let elapsed = now.elapsed();
  println!("Elapsed: {:.2?}", elapsed);
}
