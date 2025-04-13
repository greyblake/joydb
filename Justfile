all: fmt test clippy examples typos

test:
    cargo test

fmt:
    cargo fmt

clippy:
    cargo clippy -- -D warnings

examples:
    #!/usr/bin/env bash
    set -euxo pipefail
    ROOT_DIR=$(pwd)
    for EXAMPLE in `ls examples`; do
        cd $ROOT_DIR/examples/$EXAMPLE;
        if [[ "$EXAMPLE" == "todo_web_app" ]]
        then
            cargo build
        else
            cargo run
        fi
    done

typos:
    which typos >/dev/null || cargo install typos-cli
    typos
