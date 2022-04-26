/* tslint:disable */
/* eslint-disable */
/**
* @param {string} name
*/
export function greet(name: string): void;
/**
* @param {string} program
* @param {Function} cb
*/
export function syntax_tree(program: string, cb: Function): void;
/**
* @param {string} program
* @param {Function} cb
*/
export function typing(program: string, cb: Function): void;
/**
* @param {string} program
* @param {Function} cb
*/
export function one_step_eval(program: string, cb: Function): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly greet: (a: number, b: number) => void;
  readonly syntax_tree: (a: number, b: number, c: number) => void;
  readonly typing: (a: number, b: number, c: number) => void;
  readonly one_step_eval: (a: number, b: number, c: number) => void;
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
