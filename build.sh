[ -d public ] || mkdir public
wasm-pack build --target web --out-dir js/wasm
cp -r img js css index.html public/
rm public/js/wasm/.gitignore
