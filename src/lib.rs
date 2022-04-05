use wasm_bindgen::prelude::*;

pub mod error;
pub mod syntax;

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
