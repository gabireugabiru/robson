mod utils;
use std::{
  error::Error,
  fmt::Display,
  fs::{self, File},
  io::{stdin, stdout, ErrorKind, StdoutLock, Write},
  time::{Duration, Instant},
};

use crossterm::{
  cursor,
  event::{poll, read, Event, KeyCode},
  queue,
  style::Color,
  terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use robson_compiler::{
  compiler::Compiler, data_struct::IError, interpreter::Interpreter,
  Infra,
};
use utils::color_print;

pub struct RunInfra<'a> {
  stdout: StdoutLock<'a>,
}
impl<'a> RunInfra<'a> {
  pub fn new() -> Box<Self> {
    Box::new(Self {
      stdout: stdout().lock(),
    })
  }
}
impl<'a> Infra for RunInfra<'a> {
  #[inline(always)]
  fn print(&mut self, to_print: String) {
    write!(self.stdout, "{to_print}").unwrap();
  }
  fn println(&mut self, to_print: String) {
    println!("{to_print}");
  }
  fn read_line(&mut self) -> Result<String, std::io::Error> {
    let mut buffer = String::new();
    stdin().read_line(&mut buffer)?;
    Ok(buffer)
  }
  fn flush(&mut self) {
    self.stdout.flush().unwrap();
  }
  fn clear_all(&mut self) -> Result<(), IError> {
    queue!(self.stdout, Clear(ClearType::All))?;
    Ok(())
  }
  fn clear_purge(&mut self) -> Result<(), IError> {
    queue!(self.stdout, Clear(ClearType::Purge))?;
    Ok(())
  }
  fn enable_raw_mode(&self) -> Result<(), IError> {
    enable_raw_mode()?;
    Ok(())
  }
  fn disable_raw_mode(&self) -> Result<(), IError> {
    disable_raw_mode()?;
    Ok(())
  }
  fn poll(&self, duration: u64) -> Result<u32, IError> {
    let mut code = 0;
    if poll(Duration::from_millis(duration))? {
      match read()? {
        Event::Key(key) => {
          code = match &key.code {
            KeyCode::Char(a) => *a as u32,
            KeyCode::Esc => 10000,
            KeyCode::BackTab => 10001,
            KeyCode::Backspace => 10002,
            KeyCode::Delete => 10003,
            KeyCode::Down => 10004,
            KeyCode::End => 10005,
            KeyCode::Enter => 10006,
            KeyCode::Insert => 10007,
            KeyCode::Left => 10008,
            KeyCode::PageDown => 10009,
            KeyCode::PageUp => 10010,
            KeyCode::Right => 10011,
            KeyCode::Tab => 10012,
            KeyCode::Up => 10013,
            _ => 0,
          }
        }
        _ => {}
      }
    }
    Ok(code)
  }
  fn hide_cursor(&mut self) -> Result<(), IError> {
    queue!(self.stdout, cursor::Hide)?;
    Ok(())
  }
  fn show_cursor(&mut self) -> Result<(), IError> {
    queue!(self.stdout, cursor::Show)?;
    Ok(())
  }
  fn move_cursor(&mut self, x: u32, y: u32) -> Result<(), IError> {
    queue!(self.stdout, cursor::MoveTo(x as u16, y as u16))?;
    Ok(())
  }
}
fn run_compiled(
  buffer: &[u8],
  debug: bool,
  time: bool,
  lines: usize,
) -> Result<(), IError> {
  let mut interpreter = Interpreter::new(buffer, RunInfra::new())?;
  interpreter.debug(debug);

  let mut result = Ok::<(), IError>(());
  let now = Instant::now();
  loop {
    match interpreter.run_buffer() {
      Ok(a) => {
        if a {
          break;
        }
      }
      Err(interpreter_err) => {
        result = Err(interpreter_err);
        break;
      }
    }
  }

  if debug {
    print!("[");
    for i in 0..lines {
      print!("{}, ", interpreter.memory[i]);
    }
    println!("]");
  }
  if time {
    color_print(
      format!("Execution time {:.2?}", now.elapsed()),
      Color::DarkGreen,
    );
  }
  result
}

fn compile_robson(
  file_path: String,
) -> Result<Vec<u8>, Box<dyn Error>> {
  let mut compiler = Compiler::new(file_path, RunInfra::new())?;
  let buffer = compiler.compile()?;
  Ok(buffer)
}

fn warn_flags(flag: impl Display) {
  color_print(
    format!(
      "!invalid flag '{flag}', flags are:!\nDebug\nCompile\nDebug\n\nThe execution will continue ignoring it\n\n-----------------"
    ),
    Color::Yellow,
  );
}

fn write_to_file(
  buffer: &[u8],
  file_path: &str,
) -> Result<(), std::io::Error> {
  // CREATE PATH MODEL "out/NAME.rbsn"
  let mut true_path = String::from("out/");
  true_path.push_str(&file_path.replace(".robson", ".rbsn"));

  let file = File::create(&true_path);
  if let Err(err) = file {
    // CREATE OUT DIR IF IT DOESNT EXISTS
    if err.kind() == ErrorKind::NotFound {
      std::fs::create_dir("out/")?;
      let mut file = File::create(true_path)?;
      // WRITE THE BINARY
      file.write(buffer)?;
    }
  } else {
    // WRITE THE BINARY
    file.unwrap().write(buffer)?;
  }
  Ok(())
}

fn main() {
  const VERSION: &str = "0.1.2";

  let args = &std::env::args().collect::<Vec<String>>();
  let mut raw_run = true;
  let mut file_path = String::new();
  let mut debug = false;
  let mut run = false;
  let mut compile = false;
  let mut lines = 0;
  let mut time = false;
  let valid_flags = ["debug", "compile", "run", "time"];

  for (i, string) in args.iter().enumerate() {
    if i > 1 {
      let string = string.to_lowercase();
      let string = string.as_str();
      if valid_flags.contains(&string) {
        if !debug && !run && !compile && !time {
          match string {
            "debug" => {
              debug = true;
              raw_run = false
            }
            "compile" => {
              compile = true;
              raw_run = false
            }
            "run" => {
              run = true;
              raw_run = false
            }
            "time" => {
              time = true;
            }
            _ => warn_flags(string),
          }
        } else {
          color_print("!Flags other than the first are ignored!\n-----------------", Color::Yellow)
        }
      } else if debug {
        lines = match string.parse::<usize>() {
          Ok(a) => a,
          Err(_) => {
            color_print(
              format!(
                "!couldnt parse {} into integer!\n-----------------",
                string
              ),
              Color::Yellow,
            );
            0
          }
        };
      } else {
        warn_flags(string)
      }
    } else if i == 1 {
      if string.to_lowercase() == "--version" {
        println!("Robson v{}", VERSION);
        return;
      } else {
        file_path = string.to_owned();
      }
    }
  }
  //running a .rbsn file
  if raw_run {
    if !file_path.ends_with(".rbsn") {
      if file_path.ends_with(".robson") {
        color_print(format!("If you're trying to run a .robson try a `robson {file_path} [run|compile|debug]`"), Color::Yellow);
      }
      color_print("Invalid file type", Color::Red);
      return;
    }
    let buffer = match fs::read(file_path) {
      Ok(a) => a,
      Err(err) => {
        color_print(
          format!(
            "\n--------------------\n{}\n--------------------",
            err
          ),
          Color::Red,
        );
        return;
      }
    };
    if let Err(err) = run_compiled(&buffer, debug, time, lines) {
      color_print(
        format!(
          "\n--------------------\n{}\n--------------------",
          err
        ),
        Color::Red,
      );
    }

    return;
  }
  if compile || run || debug {
    //Check if it is not a robson
    if !file_path.ends_with(".robson") {
      color_print("Please select a .robson file", Color::Red);
      return;
    }
    //compile
    let now = Instant::now();
    color_print(
      format!("Compiling {}", &file_path),
      Color::DarkGreen,
    );
    let buffer = match compile_robson(file_path.clone()) {
      Ok(a) => a,
      Err(err) => {
        color_print(
          format!(
            "\n--------------------\n{}\n--------------------",
            err
          ),
          Color::Red,
        );
        return;
      }
    };
    let elapsed = now.elapsed();
    color_print(
      format!("Compiled in {:.2?}", elapsed),
      Color::DarkGreen,
    );
    //writing to file
    if let Err(err) = write_to_file(&buffer, &file_path) {
      color_print(
        format!(
          "\n--------------------\n{}\n--------------------",
          err
        ),
        Color::Red,
      );
      return;
    }

    //Run the compiled binary
    if run || debug {
      if let Err(err) = run_compiled(&buffer, debug, false, lines) {
        color_print(
          format!(
            "\n--------------------\n{}\n--------------------",
            err
          ),
          Color::Red,
        );
      }
    }
  }
}
