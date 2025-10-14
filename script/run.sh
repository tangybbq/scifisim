#! /bin/bash

# This can be run by a vscode task.  It will be passed the current file, giving
# us an opportunity to detect examples or other specific things, and decide a
# more specialized way to run.

echo "Script $@"

if [[ "$1" == *"examples/"*".rs" ]]; then
    echo "Running example $1"
    cargo run --example $(basename -s .rs "$1")
else
    echo "Running default cargo run"
    cargo run
fi
