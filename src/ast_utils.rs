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
