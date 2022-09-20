curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
npm run build
npm run start