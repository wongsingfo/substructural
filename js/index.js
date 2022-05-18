"use strict";

function hello() {
  console.log("Hello World!");
}

function debounce(func, wait, immediate) {
  var timeout;
  return function () {
    var context = this,
      args = arguments;
    var later = function () {
      timeout = null;
      if (!immediate) func.apply(context, args);
    };
    var callNow = immediate && !timeout;
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
    if (callNow) func.apply(context, args);
  };
}

function convertIndexToLineColumn(text, index) {
  let line = 0;
  let column = 0;
  for (let i = 0; i < index; i++) {
    if (text[i] === "\n") {
      line++;
      column = 0;
    } else {
      column++;
    }
  }
  return CodeMirror.Pos(line, column);
}

function parseJSON(str) {
  try {
    return JSON.parse(str);
  } catch (e) {
    return str;
  }
}

CodeMirror.defineSimpleMode("substructural", {
  // The start state contains the rules that are initially used
  start: [
    // The regex matches the token, the token property contains the type
    { regex: /"(?:[^\\]|\\.)*?(?:"|$)/, token: "string" },
    // You can match multiple tokens at once. Note that the captured
    // groups must span the whole string in this case
    // {
    // regex: /(function)(\s+)([a-z$][\w$]*)/,
    // token: ["keyword", null, "variable-2"]
    // },
    // Rules are matched in the order in which they appear, so there is
    // no ambiguity between this one and the one above
    {
      regex: /(?:bool|int|if|else)\b/,
      token: "keyword",
    },
    { regex: /true|false/, token: "atom" },
    {
      regex: /0x[a-f\d]+|[-+]?(?:\.\d+|\d+\.?\d*)(?:e[-+]?\d+)?/i,
      token: "number",
    },
    // {regex: /\/\/.*/, token: "comment"},
    // {regex: /\/(?:[^\\]|\\.)*?\//, token: "variable-3"},
    // A next property will cause the mode to move to a different state
    // {regex: /\/\*/, token: "comment", next: "comment"},
    // {regex: /[-+\/*=<>!]+/, token: "operator"},
    // indent and dedent properties guide autoindentation
    { regex: /[\{\[\(]/, indent: true },
    { regex: /[\}\]\)]/, dedent: true },
    { regex: /[a-z$][\w$]*/, token: "variable" },
    // You can embed other modes with the mode property. This rule
    // causes all code between << and >> to be highlighted with the XML
    // mode.
    // {regex: /<</, token: "meta", mode: {spec: "xml", end: />>/}}
  ],
  // The multi-line comment state.
  comment: [
    // {regex: /.*?\*\//, token: "comment", next: "start"},
    // {regex: /.*/, token: "comment"}
  ],
  // The meta property contains global information about the mode. It
  // can contain properties like lineComment, which are supported by
  // all modes, and also directives like dontIndentStates, which are
  // specific to simple modes.
  meta: {
    // dontIndentStates: ["comment"],
    // lineComment: "//"
  },
});

function Substructural() {
  let lib = window.substructural;
  let log_error = console.log;
  let editor = null;

  function init_me() {
    lib.init().then(() => {
      setTimeout(() => {
        this.initialized = true;
        init.call(this);
      }, 0);
    });
  }

  function init() {
    CodeMirror.defineMIME("substructural", "substructural");
    CodeMirror.registerHelper("lint", "substructural", (text) => {
      if (!this.initialized) {
        return;
      }
      let hints = [];

      let do_nothing = console.log;

      let do_check_typing = () => {
        lib.typing(
          text,
          (res) => {
            updateTypingOutput.call(this, res);
          },
          (err_) => {
            let err = parseJSON(err_);
            if (err.TypeError) {
              let { start, end, message } = err.TypeError;
              hints.push({
                from: convertIndexToLineColumn(text, start),
                to: convertIndexToLineColumn(text, end),
                message,
                severity: "warning",
              });
              this.typing_output = "Not well-typed: " + message;
            } else {
              console.log(err);
            }
          }
        );
      };

      lib.term_lint(text, do_check_typing, (err_) => {
        let err = parseJSON(err_);
        if (err.ParseError) {
          let { start, end, message } = err.ParseError;
          hints.push({
            from: convertIndexToLineColumn(text, start),
            to: convertIndexToLineColumn(text, end),
            message,
            severity: "error",
          });
        } else {
          console.log(err);
        }
      });

      return hints;
    });

    editor = CodeMirror.fromTextArea(document.getElementById("source-code"), {
      lineNumbers: true,
      mode: "substructural",
      tabSize: 4,
      gutters: ["CodeMirror-lint-markers"],
      lint: true,
      theme: "darcula",
    });

    editor.on("change", (_self, _obj) => {
      let code = editor.getValue();
      this.input_code = code;
      this.onInputChanged();
    });
    let code = editor.getValue();
    this.input_code = code;
    this.onInputChanged();
  }

  let eval_term1;

  function prettify(arg) {
    let result;
    lib.prettify(
      JSON.stringify(arg),
      (res) => {
        result = res;
      },
      console.error
    );
    return result;
  }

  function oneStep(arg) {
    lib.one_step_eval(
      arg,
      (res) => {
        eval_term1 = parseJSON(res);

        let term1 = eval_term1.term;
        let context1 = eval_term1.store.bindings;
        this.eval0 = this.eval1;
        this.ctx0 = this.ctx1;
        this.eval1 = prettify(term1);
        let ctx1 = [];
        for (const [key, value] of Object.entries(context1).sort((a, b) =>
          b[0].localeCompare(a[0])
        )) {
          ctx1.push([key, prettify(value)]);
        }
        this.ctx1 = ctx1;
      },
      console.error
    );
  }

  function onInputChanged() {
    let { input_code } = this;
    lib.prettify(
      input_code,
      (result) => {
        this.output_syntax = result;
        this.eval1 = result;
        this.ctx1 = [];

        oneStep.call(this, result);
      },
      log_error
    );
  }

  function onOneStepEval() {
    oneStep.call(this, JSON.stringify(eval_term1));
  }

  function updateTypingOutput(json0) {
    let json = parseJSON(json0);
    console.log(json);
    let result = json
      .map((span) => {
        let { ty, s } = span;
        if (s === "\n") {
          s = "<br/>";
        } else if (ty) {
          // TODO: esacpe quote in `ty`
          s = `<span class="typing-tip" data-text="${ty}">${s}</span>`;
        }
        return s;
      })
      .join("");

    this.typing_output = result;
  }

  function onFormatCode() {}

  function onEvalution() {}

  return {
    initialized: false,
    init_me,
    input_code: "",
    output_syntax: "",
    typing_output: "",

    onInputChanged: debounce(onInputChanged, 500),

    onFormatCode,
    onEvalution,
    onOneStepEval,

    eval0: "",
    eval1: "",
    ctx0: [],
    ctx1: [],
  };
}
