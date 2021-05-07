#!/usr/bin/env bash
set -euxo pipefail

sudo chmod +x ./e2e-artifacts/pps-cli

python3 make-build-env.py --out $HOME/build-env
export JJS_PATH=$HOME/build-env
for i in a-plus-b array-sum sqrt; do
    mkdir -p ./out/$i
    ./e2e-artifacts/pps-cli compile --pkg example-problems/$i --out ./out/$i
done
