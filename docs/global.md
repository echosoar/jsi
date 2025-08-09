
## 全局对象

### 如何创建一个全局对象

1. constants 中创建一个全局对象的名称，例如 Promise，并添加到 GLOBAL_OBJECT_NAME_LIST 列表中
    a. GLOBAL_OBJECT_NAME_LIST 列表在 context 初始化的时候，会被遍历并创建全局对象添加到全局作用域中
2. 实现 create_promise 方法，也就是用来创建 Promise 对象的方法
    a. 绑定 constructor 属性
    c. 关联原型，通过 PROTO_PROPERTY_NAME 关联 Promise.prototype
3. 实现全局构造方法 bind_global_promise
    a. 主要是设置 INSTANTIATE_OBJECT_METHOD_NAME 方法，此方法在 context 执行 new 的时候会被调用，用来实例化内置的全局对象
    b. 绑定 Promise 的静态方法
    c. 绑定 Promise.prototype 的方法(原型方法)
