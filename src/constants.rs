pub const GLOBAL_NUMBER_NAME: &str = "Number";
pub const GLOBAL_STRING_NAME: &str = "String";
pub const GLOBAL_BOOLEAN_NAME: &str = "Boolean";
pub const GLOBAL_OBJECT_NAME: &str = "Object";
pub const GLOBAL_ARRAY_NAME: &str = "Array";
pub const GLOBAL_FUNCTION_NAME: &str = "Function";
pub const GLOBAL_PROMISE_NAME: &str = "Promise";

pub const GLOBAL_ERROR_NAME: &str = "Error";
pub const GLOBAL_TYPE_ERROR_NAME: &str = "TypeError";

pub const GLOBAL_OBJECT_NAME_LIST: [&str;9] = [
    GLOBAL_NUMBER_NAME,
    GLOBAL_STRING_NAME,
    GLOBAL_BOOLEAN_NAME,
    // Object
    GLOBAL_OBJECT_NAME,
    GLOBAL_ARRAY_NAME,
    GLOBAL_FUNCTION_NAME,
    GLOBAL_PROMISE_NAME,
    // Error
    GLOBAL_ERROR_NAME,
    GLOBAL_TYPE_ERROR_NAME,
];

pub const PROTO_PROPERTY_NAME: &str = "[[Property]]";