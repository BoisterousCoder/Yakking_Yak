echo Yakking Yak: installing rust
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
echo installing wasm
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

echo Yakking Yak: creating public/bin
cd public
mkdir bin
cd ..

echo Yakking Yak: building wasm
wasm-pack build --target web
echo Yakking Yak: building js
npx browserify src/js/start.js -o public/bin/main.js
echo Yakking Yak: building html
npx pug ./src/pug -o ./public

echo Yakking Yak: copying wasm files to public bin
cp ./pkg/yakking_yak.js ./public/bin/yakking_yak.js
cp ./pkg/yakking_yak_bg.wasm ./public/bin/yakking_yak_bg.wasm

echo Yakking Yak: copying css file to public bin
cp ./src/css/style.css ./public/bin/style.css

echo Yakking Yak: listing public folder
cd public
ls

echo Yakking Yak: Done!