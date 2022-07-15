use std::{fmt::Display, io::stdout};

use crossterm::{
  execute,
  style::{Color, Print, ResetColor, SetForegroundColor},
};

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
