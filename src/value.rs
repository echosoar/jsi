#[derive(Debug)]
pub enum Value {
  String(String),
  Number(f64),
  Boolean(bool),
  Null,
  Undefined,
  NAN,
}

impl Value {
  pub fn is_string(&self) -> bool {
    if let Value::String(_) = self {
      return true
    }
    return false
  }
  pub fn to_string(&self) -> String {
    match self {
      Value::String(str) => str.clone(),
      Value::Number(number) => number.to_string(),
      Value::Boolean(bool) => {
        if *bool {
          String::from("true")
        } else {
          String::from("false")
        }
      },
      Value::NAN => String::from("NaN"),
      _ => String::from(""),
    }
  }
  pub fn is_number(&self) -> bool {
    if let Value::Number(_) = self {
      return true
    }
    return false
  }

  pub fn is_infinity(&self) -> bool {
    if let Value::Number(number) = self {
      return *number == f64::INFINITY || *number == -f64::INFINITY;
    }
    return false
  }

  pub fn is_nan(&self) -> bool {
    if let Value::NAN = self {
      return true
    }
    return false
  }

  pub fn to_number(&self) -> f64 {
    match self {
      Value::String(str) => str.parse::<f64>().unwrap(),
      Value::Number(number) => *number,
      Value::Boolean(bool) => {
        if *bool {
          1f64
        } else {
          0f64
        }
      },
      _ => 0f64,
    }
  }
  pub fn is_boolean(&self) -> bool {
    if let Value::Boolean(_) = self {
      return true
    }
    return false
  }

}