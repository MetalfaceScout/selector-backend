SRC:= src/db.rs src/main.rs src/output.rs src/selection.rs src/stats.rs

all: build

build: $(SRC)
	cargo build

install: $(SRC)
	sudo cargo install --path ./ --root /usr/local/