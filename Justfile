all: build

build: css
    @cargo build

css:
    @mkdir -p resources
    @cp -r images/* resources
    @sassc -t compressed sass/style.sass resources/style.min.css

run: build
    @RUST_LOG=info ./target/debug/valentine web

clean:
    rm -r resources target valentine.tgz

build-release: css
    @cargo build --release

run-release: build-release
    @cargo run --release -- web

tar: build-release
    @tar -cf valentine.tgz target/release/valentine resources
