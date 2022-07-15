pub fn u32_add(a: [u8; 4], b: [u8; 4]) -> u32 {
  u32::from_be_bytes(a) + u32::from_be_bytes(b)
}

pub fn f32_add(a: [u8; 4], b: [u8; 4]) -> f32 {
  (f32::from_be_bytes(a) + f32::from_be_bytes(b)) as f32
}

pub fn i32_add(a: [u8; 4], b: [u8; 4]) -> i32 {
  i32::from_be_bytes(a) + i32::from_be_bytes(b)
}

pub fn u32_sub(a: [u8; 4], b: [u8; 4]) -> u32 {
  u32::from_be_bytes(a) - u32::from_be_bytes(b)
}

pub fn f32_sub(a: [u8; 4], b: [u8; 4]) -> f32 {
  f32::from_be_bytes(a) - f32::from_be_bytes(b)
}

pub fn i32_sub(a: [u8; 4], b: [u8; 4]) -> i32 {
  i32::from_be_bytes(a) - i32::from_be_bytes(b)
}
pub fn u32_mul(a: [u8; 4], b: [u8; 4]) -> u32 {
  u32::from_be_bytes(a) * u32::from_be_bytes(b)
}

pub fn f32_mul(a: [u8; 4], b: [u8; 4]) -> f32 {
  f32::from_be_bytes(a) * f32::from_be_bytes(b)
}

pub fn i32_mul(a: [u8; 4], b: [u8; 4]) -> i32 {
  i32::from_be_bytes(a) * i32::from_be_bytes(b)
}

pub fn u32_div(a: [u8; 4], b: [u8; 4]) -> u32 {
  u32::from_be_bytes(a) / u32::from_be_bytes(b)
}

pub fn f32_div(a: [u8; 4], b: [u8; 4]) -> f32 {
  f32::from_be_bytes(a) / f32::from_be_bytes(b)
}

pub fn i32_div(a: [u8; 4], b: [u8; 4]) -> i32 {
  i32::from_be_bytes(a) / i32::from_be_bytes(b)
}

pub fn approx_equal(a: f32, b: f32, decimal_places: u8) -> bool {
  let factor = 10.0f32.powi(decimal_places as i32);
  let a = (a * factor).trunc();
  let b = (b * factor).trunc();
  a == b
}
