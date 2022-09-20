curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
wasm-pack build --target web
cd ./pkg
ls
cd ..
cp -r ./pkg/yakking_yak.js ./public/bin/yakking_yak.js
cp -r ./pkg/yakking_yak_bg.wasm ./public/bin/yakking_yak_bg.wasm
npx browserify src/js/start.js -o public/bin/main.js
npx pug ./src/pug -o ./public
cp -r ./src/css/style.css ./public/bin/style.css