use wasm_bindgen::prelude::*;

pub mod error;
pub mod syntax;
pub mod formatter;
pub mod typing;
pub mod eval;

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen]
pub fn syntax_tree(program: &str, cb: &js_sys::Function) {
    let tree = syntax::parse_program(program);
    let result = match tree {
        Ok(tree) => format!("{:#?}", tree),
        Err(error) => format!("{}", error),
    };
    let result = JsValue::from_str(&result);
    let this = JsValue::NULL;
    let _ = cb.call1(&this, &result);
}

#[wasm_bindgen]
pub fn typing(program: &str, cb: &js_sys::Function) {
    let result = "todo(crz)";
    let result = JsValue::from_str(&result);
    let this = JsValue::NULL;
    let _ = cb.call1(&this, &result);
}

#[wasm_bindgen]
pub fn one_step_eval(program: &str, cb: &js_sys::Function) {
    let result = "todo(wck)";
    let result = JsValue::from_str(&result);
    let this = JsValue::NULL;
    let _ = cb.call1(&this, &result);
}
