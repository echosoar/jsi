pub fn get_hex_number_value(chr: char) -> i32 {
  match chr {
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
    'a' | 'A' => 10,
    'b' | 'B' => 11,
    'c' | 'C' => 12,
    'd' | 'D' => 13,
    'e' | 'E' => 14,
    'f' | 'F' => 15,
    _ => 16,
  }
}

pub fn chars_to_string(chars: &Vec<char>, start: usize, end: usize) -> String {
  return chars[start..end].iter().collect()
}

// 处理字符串转义序列
pub fn process_string_escapes(s: &str) -> String {
  let mut result = String::new();
  let mut chars = s.chars().peekable();

  while let Some(c) = chars.next() {
    if c == '\\' {
      if let Some(next) = chars.next() {
        match next {
          'n' => result.push('\n'),
          't' => result.push('\t'),
          'r' => result.push('\r'),
          '\\' => result.push('\\'),
          '\'' => result.push('\''),
          '"' => result.push('"'),
          '0' => result.push('\0'),
          'x' => {
            // \xHH 十六进制转义
            let hex1 = chars.next();
            let hex2 = chars.next();
            if let (Some(h1), Some(h2)) = (hex1, hex2) {
              let val = (get_hex_number_value(h1) << 4) + get_hex_number_value(h2);
              if let Some(ch) = char::from_u32(val as u32) {
                result.push(ch);
              }
            }
          },
          'u' => {
            // \uHHHH Unicode 转义
            let mut val = 0;
            for _ in 0..4 {
              if let Some(h) = chars.next() {
                val = (val << 4) + get_hex_number_value(h);
              }
            }
            if let Some(ch) = char::from_u32(val as u32) {
              result.push(ch);
            }
          },
          _ => {
            // 未知的转义序列，保留原样
            result.push('\\');
            result.push(next);
          }
        }
      }
    } else {
      result.push(c);
    }
  }

  result
}
