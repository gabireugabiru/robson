mod utils;
use std::{
  fmt::Display,
  fs::{self, File},
  io::{stdin, stdout, ErrorKind, StdoutLock, Write},
  path::Path,
  time::{Duration, Instant},
};

use crossterm::{
  cursor,
  event::{poll, read, Event, KeyCode},
  queue,
  style::{Color, Print, ResetColor, SetForegroundColor},
  terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use robson_compiler::{
  compiler::Compiler,
  data_struct::IError,
  interpreter::{self, Interpreter},
  Infra,
};
use utils::{color_print, color_print_no_newline};

use crate::utils::print_err;
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
  fn clone_self(&mut self) -> Box<dyn Infra> {
    Box::new(RunInfra {
      stdout: stdout().lock(),
    })
  }
  fn color_print(&mut self, string: String, color: u32) {
    if queue!(
      self.stdout,
      SetForegroundColor(Color::AnsiValue(color as u8)),
      Print(format!("{}", string)),
      ResetColor
    )
    .is_err()
    {
      print!("{}", string);
    }
  }
}

fn change_extension(path: String) -> String {
  let mut new_path = String::new();
  let splited: Vec<&str> = path.split(".robson").collect();
  for (i, str) in splited.iter().enumerate() {
    if i != splited.len() - 1 {
      new_path.push_str(str);
    }
  }
  new_path.push_str(".rbsn");
  new_path
}

fn run_compiled(
  buffer: &[u8],
  debug: bool,
  time: bool,
) -> Result<(), IError> {
  let mut interpreter = Interpreter::new(buffer, RunInfra::new())?;
  interpreter.debug(debug);

  // let mut result = Ok::<(), IError>(());
  let now = Instant::now();

  let result = interpreter.run_buffer();

  if debug {
    print!("[");
    for i in interpreter.memory {
      print!("{}, ", i);
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

fn compile_robson(file_path: String) -> Result<Vec<u8>, IError> {
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
  let file_path = change_extension(file_path.to_owned());
  let path = Path::new(&file_path);

  let parent = path.parent().ok_or(std::io::Error::new(
    ErrorKind::NotFound,
    "Specified path nas no parent",
  ))?;

  let parent_string = parent.to_string_lossy();

  let parent_string = if parent_string.is_empty() {
    "".to_owned()
  } else {
    format!("{}/", parent.to_string_lossy())
  };

  let true_path = format!(
    "{parent_string}out/{}",
    path
      .file_name()
      .ok_or(std::io::Error::new(
        ErrorKind::InvalidInput,
        "Expected a file"
      ))?
      .to_str()
      .unwrap()
  );
  println!("{}", true_path);

  let file = File::create(&true_path);
  if let Err(err) = file {
    // CREATE OUT DIR IF IT DOESNT EXISTS
    if err.kind() == ErrorKind::NotFound {
      match std::fs::create_dir(format!("{parent_string}out")) {
        Ok(_) => {}
        Err(err) => {
          if err.kind() != ErrorKind::AlreadyExists {
            return Err(err);
          }
        }
      };
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
  const VERSION: &str = "0.1.4";
  let args = &std::env::args().collect::<Vec<String>>();
  let mut raw_run = true;
  let mut file_path = String::new();
  let mut debug = false;
  let mut run = false;
  let mut compile = false;
  let mut time = false;
  let mut print = false;
  let valid_flags = ["debug", "compile", "run", "time", "print"];

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
            "print" => {
              print = true;
            }
            _ => warn_flags(string),
          }
        } else {
          color_print("!Flags other than the first are ignored!\n-----------------", Color::Yellow)
        }
      } else {
        warn_flags(string)
      }
    } else if i == 1 {
      match string.to_lowercase().as_str() {
        "--version" => {
          const VALID_CHARS: [char; 11] =
            ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.'];

          println!("");

          color_print(
            format!("{}", include_str!("asciilogo")),
            Color::Magenta,
          );

          let numbers = vec![
            include_str!("numbers/0"),
            include_str!("numbers/1"),
            include_str!("numbers/2"),
            include_str!("numbers/3"),
            include_str!("numbers/4"),
            include_str!("numbers/5"),
            include_str!("numbers/6"),
            include_str!("numbers/7"),
            include_str!("numbers/8"),
            include_str!("numbers/9"),
            include_str!("dot"),
          ];
          let mut line_numbers: Vec<Vec<&str>> = Vec::new();
          for i in numbers {
            line_numbers.push(i.lines().collect());
          }
          println!("");
          for i in 0..9 {
            for char in VERSION.chars() {
              if VALID_CHARS.contains(&char) {
                let value: usize = match char {
                  '0' => 0,
                  '1' => 1,
                  '2' => 2,
                  '3' => 3,
                  '4' => 4,
                  '5' => 5,
                  '6' => 6,
                  '7' => 7,
                  '8' => 8,
                  '9' => 9,
                  _ => 10,
                };

                color_print_no_newline(
                  format!("  {}", line_numbers[value][i]),
                  Color::Magenta,
                );
              }
            }
            print!("\n");
          }
          println!("");

          return;
        }
        "--generate" => {
          let mut string = String::new();

          match stdin().read_line(&mut string) {
            Ok(_) => {}
            Err(err) => print_err(err.into()),
          };
          let mut chars: Vec<char> = string.chars().collect();

          chars.reverse();
          println!("robson robson robson");
          for i in chars {
            println!("comeu {}", i as u32);
          }
          return;
        }
        "--chars" => {
          let int = interpreter::Interpreter::new(
            include_bytes!("out/chars.rbsn"),
            RunInfra::new(),
          );
          if int.is_err() {
            color_print("Something went wrong", Color::Red);
            return;
          }
          let mut int = int.unwrap();
          if let Err(err) = int.run_buffer() {
            print_err(err);
          }
          return;
        }
        _ => {
          file_path = string.to_owned();
        }
      }
    }
  }
  //running a .rbsn file
  if raw_run && !print {
    if !file_path.ends_with(".rbsn") {
      if file_path.ends_with(".robson") {
        color_print(format!("If you're trying to run a .robson try a `robson {file_path} [run|compile|debug]`"), Color::Yellow);
      } else if file_path.starts_with("--") {
        color_print("Invalid flag command, valids are:\n--generate\n--version\n--char\n\n", Color::Yellow)
      }
      color_print("Invalid file type", Color::Red);
      return;
    }
    let buffer = match fs::read(file_path) {
      Ok(a) => a,
      Err(err) => {
        print_err(err.into());
        return;
      }
    };
    if let Err(err) = run_compiled(&buffer, debug, time) {
      print_err(err);
    }
    return;
  }

  if print {
    if !file_path.ends_with(".rbsn") {
      color_print(
        "Trying to print a invalid buffer, please select a .rbsn",
        Color::Yellow,
      );
      return;
    }
    let buffer = match fs::read(file_path) {
      Ok(a) => a,
      Err(err) => {
        print_err(err.into());
        return;
      }
    };
    robson_compiler::print_file_buffer(buffer);
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
        print_err(err);
        return;
      }
    };
    let elapsed = now.elapsed();
    color_print(format!("Finished in {:.2?}", elapsed), Color::Green);
    //writing to file
    if let Err(err) = write_to_file(&buffer, &file_path) {
      print_err(err.into());
      return;
    }

    //Run the compiled binary
    if run || debug {
      color_print(format!("Running now"), Color::Magenta);
      if let Err(err) = run_compiled(&buffer, debug, false) {
        print_err(err)
      }
    }
  }
}
