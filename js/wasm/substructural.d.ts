/* tslint:disable */
/* eslint-disable */
/**
* @param {string} name
*/
export function greet(name: string): void;
/**
* Lint the given source code.
*
* # Arguments
* * `source` - The source code to lint.
* @param {string} program
* @param {Function} cb_ok
* @param {Function} cb_err
*/
export function term_lint(program: string, cb_ok: Function, cb_err: Function): void;
/**
* @param {string} program
* @param {Function} cb
*/
export function typing(program: string, cb: Function): void;
/**
* Evaluate a program.
*
* # Arguments
* * `term_eval_json` - The program to evaluate. It can be the `TermEval`, the `TermCtx`, or the
* source code in string form. The latter two are converted to `TermEval` with an empty store
* before evaluation.
* @param {string} program
* @param {Function} cb_ok
* @param {Function} cb_err
*/
export function one_step_eval(program: string, cb_ok: Function, cb_err: Function): void;
/**
* Prettify the term
*
* # Arguments
* * `term_json` - The term to prettify. It can be the `TermCtx` or the source code in string form.
* The latter is converted to `TermCtx` before prettifying.
* @param {string} term_ctx
* @param {Function} cb_ok
* @param {Function} cb_err
*/
export function prettify(term_ctx: string, cb_ok: Function, cb_err: Function): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly greet: (a: number, b: number) => void;
  readonly term_lint: (a: number, b: number, c: number, d: number) => void;
  readonly typing: (a: number, b: number, c: number) => void;
  readonly one_step_eval: (a: number, b: number, c: number, d: number) => void;
  readonly prettify: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
