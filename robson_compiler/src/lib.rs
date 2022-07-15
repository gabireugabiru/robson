pub mod utils;
pub mod interpreter;
#[cfg(test)]
mod tests;

pub trait Infra {
  fn read_line(&self) -> Result<String, std::io::Error>;
  fn print(&mut self, to_print: String);
  fn println(&mut self, to_print: String);
}

