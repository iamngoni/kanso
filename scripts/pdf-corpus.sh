#!/bin/sh
# Writes the hostile-PDF corpus (issue #43, step 9) to a folder, to look at
# or to try in another reader:
#
#   scripts/pdf-corpus.sh [folder]        # default: ./pdf-corpus
#
# Every file is built in code (crates/kanso-pdf/src/hostile.rs); nothing is
# downloaded. `cargo test -p kanso-pdf --features render` runs them all
# through the real worker.
set -eu

cd "$(dirname "$0")/.."
dir=${1:-pdf-corpus}
cargo run --quiet --locked -p kanso-pdf --features test-support --example pdf_corpus -- pdfs "$dir"
