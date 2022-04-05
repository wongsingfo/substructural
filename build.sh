[ -d public ] || mkdir public
wasm-pack build --target web --out-dir js/wasm
cp -r js css index.html public/
rm public/js/wasm/.gitignore
