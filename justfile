# https://just.systems

model := "efficientnet-lite4.onnx"
model_cfg := "efficientnet-lite4.cfg"

# model := "mobilenetv2.onnx"
# model_cfg := "mobilenetv2.cfg"
#
# model := "inception-v2.onnx"
# model_cfg := "inception-v2.cfg"
# model := "xenova_convnext-tiny-224_fp32.onnx"
# model_cfg := "convnext-fp32.cfg"

default:
    just --list

set-model:
    cp dev/{{ model }} src/ml/model.onnx
    cp dev/{{ model_cfg }} src/ml/model.cfg

build:
    cargo build

run id='':
    cargo run -- {{ id }}

fmt:
    cargo fmt --all

lint:
    cargo clippy --all-targets --all-features -- -D warnings

# use cargo audit from rustsec to find vulnerabilities
audit:
    cargo audit

# use the mozilla cargo vet to protect from supply-chain attacks
vet:
    cargo vet

# run fmt, lint, audit and vet
ci: fmt lint audit vet

release: ci
    cargo build --release

clean:
    rm -rf docs/*
    cargo clean

dev-serve:
    @echo "Open http://127.0.0.1:8080/netthinne/"
    @dx serve

serve:
    @mkdir -p /tmp/netthinne-serve
    @ln -sfn "$(pwd)/docs" /tmp/netthinne-serve/netthinne
    @echo "Open http://localhost:8080/netthinne/"
    @python3 -m http.server 8080 --directory /tmp/netthinne-serve

# build/publish using dx build
# targeting  release and web then copy from target/ to docs/

# finally, optimize the wasm build with wasm-opt
publish: clean set-model
    RUSTFLAGS="-C target-feature=+simd128" dx build --release --platform web --debug-symbols false
    cp -r target/dx/netthinne/release/web/public/. docs/
    python3 scripts/dehash_assets.py docs/
    find docs -name "*.wasm" -exec wasm-opt -O3 --enable-simd --enable-bulk-memory --strip-debug --strip-producers {} -o {} \;
    python3 scripts/inject_pwa_tags.py docs/index.html
    touch docs/.nojekyll

# bump version (patch, minor, major)
bump kind="patch":
    cargo set-version --bump {{ kind }}
