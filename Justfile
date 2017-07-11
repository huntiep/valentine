all: build

build: css
    @cargo build

css:
    @mkdir -p resources
    @sassc -t compressed sass/style.sass resources/style.min.css

run: build
    @RUST_LOG=info ./target/debug/valentine web

clean:
    rm -r resources target valentine.tgz

drop-tables:
    @./drop_tables.sh

build-release: css
    @cargo build --release

run-release: build-release
    @cargo run --release

tar: build-release
    @tar -cf valentine.tgz target/release/valentine resources
