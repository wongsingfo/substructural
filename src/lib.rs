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

/// Lint the given source code.
///
/// # Arguments
/// * `source` - The source code to lint.
#[wasm_bindgen]
pub fn term_lint(program: &str, cb_ok: &js_sys::Function, cb_err: &js_sys::Function) -> Result<(), JsValue> {
    let this = JsValue::NULL;
    let tree = syntax::parse_program(program);
    let result = match tree {
        Ok(tree) => tree,
        Err(error) => {
            let error = serde_json::to_string(&error).unwrap();
            cb_err.call1(&this, &JsValue::from_str(&error)).unwrap();
            return Ok(());
        }
    };
    let result = formatter::format_termctx(&result);
    let result = JsValue::from_str(&result);
    let this = JsValue::NULL;
    cb_ok.call1(&this, &result)?;
    Ok(())
}

#[wasm_bindgen]
pub fn typing(program: &str, cb_ok: &js_sys::Function, cb_err: &js_sys::Function) {
    let this = JsValue::NULL;
    let term = match serde_json::from_str::<syntax::TermCtx>(program) {
        Ok(term) => term,
        Err(_error) => match syntax::parse_program(program) {
            Ok(term) => term,
            Err(_error) => {
                let error = error::Error::InternalError {
                    message: "Failed to parse program for typing".to_string(),
                };
                let error = serde_json::to_string(&error).unwrap();
                let error = JsValue::from_str(&error);
                cb_err.call1(&this, &error).unwrap();
                return;
            }
        },
    };
    let _result = match typing::type_check(&term) {
        Ok(result) => result,
        Err(error) => {
            let error = serde_json::to_string(&error).unwrap();
            cb_err.call1(&this, &JsValue::from_str(&error)).unwrap();
            return;
        }
    };
    // Reformat the code and do the typing again.
    let term_s = formatter::format_termctx(&term);
    let term = syntax::parse_program(&term_s).unwrap();
    let result = typing::type_check(&term);
    let result = match result {
        Ok(result) => result,
        Err(error) => {
            let error = serde_json::to_string(&error).unwrap();
            cb_err.call1(&this, &JsValue::from_str(&error)).unwrap();
            return;
        }
    };
    let result = typing::convert_hashmap_to_vec(&result, &term_s);
    let result = match serde_json::to_string(&result) {
        Ok(result) => result,
        Err(error) => {
            let error = error::Error::InternalError {
                message: error.to_string(),
            };
            let error = serde_json::to_string(&error).unwrap();
            cb_err.call1(&this, &JsValue::from_str(&error)).unwrap();
            return;
        }
    };
    let result = JsValue::from_str(&result);
    let _ = cb_ok.call1(&this, &result).unwrap();
}

/// Evaluate a program.
///
/// # Arguments
/// * `term_eval_json` - The program to evaluate. It can be the `TermEval`, the `TermCtx`, or the
/// source code in string form. The latter two are converted to `TermEval` with an empty store
/// before evaluation.
#[wasm_bindgen]
pub fn one_step_eval(program: &str, cb_ok: &js_sys::Function, cb_err: &js_sys::Function) {
    let this = JsValue::NULL;
    let term_eval = match serde_json::from_str::<eval::TermEval>(program) {
        Ok(term_eval) => term_eval,
        Err(_error) => match serde_json::from_str::<syntax::TermCtx>(program) {
            Ok(term) => Into::<eval::TermEval>::into(term),
            Err(_error) => match syntax::parse_program(program) {
                Ok(term) => Into::<eval::TermEval>::into(term),
                Err(_error) => {
                    let error = error::Error::InternalError {
                        message: "Failed to parse program for evaluation".to_string(),
                    };
                    let error = serde_json::to_string(&error).unwrap();
                    let error = JsValue::from_str(&error);
                    cb_err.call1(&this, &error).unwrap();
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
            cb_err.call1(&this, &error).unwrap();
            return;
        }
    };
    let result = serde_json::to_string(&result).unwrap();
    let result = JsValue::from_str(&result);
    let _ = cb_ok.call1(&this, &result);
}

/// Prettify the term
///
/// # Arguments
/// * `term_json` - The term to prettify. It can be the `TermCtx` or the source code in string form.
/// The latter is converted to `TermCtx` before prettifying.
#[wasm_bindgen]
pub fn prettify(term_ctx: &str, cb_ok: &js_sys::Function, cb_err: &js_sys::Function) {
    let this = JsValue::NULL;
    let term_ctx = match serde_json::from_str::<syntax::TermCtx>(term_ctx) {
        Ok(term_ctx) => term_ctx,
        Err(_error) => match syntax::parse_program(term_ctx) {
            Ok(term) => Into::<syntax::TermCtx>::into(term),
            Err(_error) => {
                let error = error::Error::InternalError {
                    message: "Failed to parse term for prettifying".to_string(),
                };
                let error = serde_json::to_string(&error).unwrap();
                let error = JsValue::from_str(&error);
                cb_err.call1(&this, &error).unwrap();
                return;
            }
        },
    };
    let result = formatter::format_termctx(&term_ctx);
    let result = JsValue::from_str(&result);
    let _ = cb_ok.call1(&this, &result);
}
