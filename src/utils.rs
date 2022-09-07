use std::{fmt::Display, io::stdout};

use crossterm::{
  execute,
  style::{Color, Print, ResetColor, SetForegroundColor},
};
use robson_compiler::data_struct::IError;

pub fn color_print<T>(string: T, color: Color)
where
  T: Display,
{
  if execute!(
    stdout(),
    SetForegroundColor(color),
    Print(format!("{}\n", string)),
    ResetColor
  )
  .is_err()
  {
    println!("{}", string);
  }
}
pub fn color_print_no_newline<T>(string: T, color: Color)
where
  T: Display,
{
  if execute!(
    stdout(),
    SetForegroundColor(color),
    Print(format!("{}", string)),
    ResetColor
  )
  .is_err()
  {
    print!("{}", string);
  }
}

pub fn print_err(err: IError) {
  color_print(format!("\nError\n{}", err), Color::Red);
}
