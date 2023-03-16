## JSI
JSI is a JavaScript Interpreter written in Rust.
---

### Development
+ git submodule
+ test262 `RUST_MIN_STACK=8388608 cargo test --package jsi --test test262_test -- test_all_262 --exact --nocapture`

### Refs
+ Ecma Standard: https://tc39.es/ecma262/multipage/#sec-intro
+ Test262: https://github.com/tc39/test262

---
by [echosoar](https://github.com/echosoar)