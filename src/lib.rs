use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet() -> String {
  "Hello, {{project-name}}!".to_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    assert_eq!(greet(), "Hello, {{project-name}}!".to_string())
  }
}
