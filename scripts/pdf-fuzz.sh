#!/bin/sh
# Fuzzes the PDF worker's protocol decoder (issue #43, step 9) with
# cargo-fuzz: valid frames, and the hostile corpus as the PDFs they carry,
# are the seeds.
#
#   scripts/pdf-fuzz.sh [seconds]         # default 60
#
# Needs a nightly toolchain and `cargo install cargo-fuzz`. Crashes land in
# crates/kanso-pdf/fuzz/artifacts/frame/ and fail the run; replay one with
# `cd crates/kanso-pdf && cargo +nightly fuzz run frame <file>`.
set -eu

cd "$(dirname "$0")/.."
seconds=${1:-60}

cargo run --quiet --locked -p kanso-pdf --features test-support --example pdf_corpus -- \
    frames crates/kanso-pdf/fuzz/corpus/frame
cd crates/kanso-pdf
# rss_limit_mb: the decoder allocates nothing it was not sent, so a run that
# grows past this has found a bug. max_len covers a frame head, its JSON and a
# small PDF.
exec cargo +nightly fuzz run frame -- \
    -max_total_time="$seconds" -rss_limit_mb=1024 -max_len=131072
