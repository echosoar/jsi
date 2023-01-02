#[derive(Debug)]
pub struct Object {
  // 原型链
  _prototype: Box<Object>,
}
