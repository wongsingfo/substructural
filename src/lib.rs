use wasm_bindgen::prelude::*;

pub mod error;
pub mod eval;
pub mod formatter;
pub mod syntax;
pub mod typing;

#[wasm_bindgen]
extern "C" {
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

/// Evaluate a program.
///
/// # Arguments
/// * `term_eval_json` - The program to evaluate. It can be the `TermEval`, the `TermCtx`, or the
/// source code in string form. The latter two are converted to `TermEval` with an empty store
/// before evaluation.
/// * `cb` - A callback function to receive the result. The first argument is the result. If error
/// occurs, the first argument is null and the second argument is the error message.
#[wasm_bindgen]
pub fn one_step_eval(program: &str, cb: &js_sys::Function) {
    let this = JsValue::NULL;
    let term_eval = match serde_json::from_str::<eval::TermEval>(program) {
        Ok(term_eval) => term_eval,
        Err(_error) => match serde_json::from_str::<syntax::TermCtx>(program) {
            Ok(term) => Into::<eval::TermEval>::into(term),
            Err(_error) => match syntax::parse_program(program) {
                Ok(term) => Into::<eval::TermEval>::into(term),
                Err(_error) => {
                    let error = error::Error::InternalError;
                    let error = serde_json::to_string(&error).unwrap();
                    let error = JsValue::from_str(&error);
                    cb.call2(&this, &JsValue::NULL, &error).unwrap();
                    return;
                }
            },
        },
    };
    let result = match eval::one_step_eval(term_eval) {
        Ok(result) => result,
        Err(error) => {
            let error = serde_json::to_string(&error).unwrap();
            let error = JsValue::from_str(&error);
            cb.call2(&this, &JsValue::NULL, &error).unwrap();
            return;
        }
    };
    let result = serde_json::to_string(&result).unwrap();
    let result = JsValue::from_str(&result);
    let _ = cb.call1(&this, &result);
}

/// Prettify the term
///
/// # Arguments
/// * `term_json` - The term to prettify. It can be the `TermCtx` or the source code in string form.
/// The latter is converted to `TermCtx` before prettifying.
/// * `cb` - A callback function to receive the result. The first argument is the result. If error
/// occurs, the first argument is null and the second argument is the error message.
#[wasm_bindgen]
pub fn prettify(term_ctx: &str, cb: &js_sys::Function) {
    let this = JsValue::NULL;
    let term_ctx = match serde_json::from_str::<syntax::TermCtx>(term_ctx) {
        Ok(term_ctx) => term_ctx,
        Err(_error) => {
            match syntax::parse_program(term_ctx) {
                Ok(term) => Into::<syntax::TermCtx>::into(term),
                Err(_error) => {
                    let error = error::Error::InternalError;
                    let error = serde_json::to_string(&error).unwrap();
                    let error = JsValue::from_str(&error);
                    cb.call2(&this, &JsValue::NULL, &error).unwrap();
                    return;
                }
            }
        }
    };
    let result = formatter::format_termctx(&term_ctx);
    let result = JsValue::from_str(&result);
    let _ = cb.call1(&this, &result);
}

