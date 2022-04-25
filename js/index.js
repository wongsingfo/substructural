"use strict";

function hello() {
	console.log('Hello World!');
}

function debounce(func, wait, immediate) {
	var timeout;
	return function () {
		var context = this, args = arguments;
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

function Substructural() {
	let lib = window.substructural;

	let input_code = '|x: $($(bool) -> ($bool)) -> int| y (z)';

	function init() {
		lib.init().then(setTimeout(() => {
			this.initialized = true;
		}, 0));
	}

	function onInputChanged() {
		let {input_code} = this;
		lib.syntax_tree(input_code, (result) => {
			this.output_syntax = result;
		});
	}

	return {
		initialized: false,
		init,
		input_code,
		output_syntax: '',

		onInputChanged: debounce(onInputChanged, 500),
	};
}
