#!/usr/bin/env bash
set -euxo pipefail

mkdir e2e-artifacts
cargo install --path cli
cp ~/.cargo/bin/pps-cli e2e-artifacts/pps-cli