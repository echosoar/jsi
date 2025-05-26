## JSI

JSI is a JavaScript interpreter written in Rust that supports bytecode similar to quickjs.


<img src="https://img.shields.io/badge/Test262-4280%20Passed-brightgreen.svg" alt="test262 passed" />
<img src="https://img.shields.io/badge/Test262-46442%20Total-blue.svg" alt="test262 total" />

---

### Usage
```rust
use jsi::JSI;

let mut jsi = JSI::new();
let result = jsi.run(String::from("\
  let a = [];
  let i = 0;
  outer:
  while(i < 3) {
    i ++;
    let j = 0;
    while(j < 5) {
      j ++;
      if (j == 1 && i == 1) {
        continue outer
      }
      if (j == 4) break
      if (j == 3 && i == 2) {
        break outer
      }
      a.push(i * j);
    }
  }
  a.join(':')")
).unwrap();
assert_eq!(result , Value::String(String::from("2:4")));
```

### Development

+ git submodule `git submodule update --init --recursive`
+ test262 `RUST_MIN_STACK=8388608 cargo test --package jsi --test test262_test -- test_all_262 --exact --nocapture`

### Refs

+ Ecma Standard: https://tc39.es/ecma262/multipage/#sec-intro
+ Test262: https://github.com/tc39/test262

### License
MIT

---
by [echosoar](https://github.com/echosoar)