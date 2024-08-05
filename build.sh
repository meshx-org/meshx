cargo build -p component_manager --target wasm32-wasi --release
wasi2fx target/wasm32-wasi/release/component_manager.wasm component_manager.wasm
wasm-opt --asyncify --pass-arg=asyncify-imports@fiber.fx_port_wait -Oz component_manager.wasm --enable-bulk-memory -o component_manager.wasm
wasm-strip component_manager.wasm -o stripped.wasm
cp component_manager.wasm ./experiments/fiber-host/public/output.wasm