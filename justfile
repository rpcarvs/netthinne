# https://just.systems

default:
    just --list

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
    rm -rf dist/*
    cargo clean

dev-serve:
    dx serve

serve:
    python3 -m http.server 8080 --directory dist

# build/publish using dx build
# targeting  release and web then copy from target/ to dist/

# finally, optimize the wasm build with wasm-opt
publish:
    RUSTFLAGS="-C target-feature=+simd128" dx build --release --platform web --debug-symbols false
    cp -r target/dx/netthinne/release/web/public/. dist/
    find dist -name "*.wasm" -exec wasm-opt -O3 --enable-simd --strip-dwarf {} -o {} \;
    touch dist/.nojekyll

# bump version (patch, minor, major)
bump kind="patch":
    cargo set-version --bump {{ kind }}
