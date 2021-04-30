#!/usr/bin/env python3

import argparse
import tempfile
import os
import subprocess
import shutil


def main(args):
    print(args)
    os.makedirs(f"{args.out}/bin", exist_ok = True)
    if "svaluer" in args.filter:
        print("Building svaluer")
        env = dict(**os.environ)
        env["RUSTC_BOOTSTRAP"] = "1"
        subprocess.run(["cargo", "build", "-p", "svaluer", "-Zunstable-options", "--out-dir", f"{args.tmp}"], env=env, check=True)
        shutil.copy(f"{args.tmp}/svaluer", f"{args.out}/bin/svaluer")
    if "jtl" in args.filter:
        print("Building JTL")
        subprocess.run(["cmake", "-S", f"{args.source}/jtl", "-B", f"{args.tmp}/cmake", f"-DCMAKE_INSTALL_PREFIX={args.out}"], check=True)
        subprocess.run(["cmake", "--build", f"{args.tmp}/cmake"], check=True)
        subprocess.run(["cmake", "--install", f"{args.tmp}/cmake"], check=True)

DESCRIPTION = '''
Script that compiles PPS problem build environment.

This environment contains files that are used when compiling or
importing problems.
'''

parser = argparse.ArgumentParser(description=DESCRIPTION)
parser.add_argument('--source', type=str,
                    help='path to jjs-pps repository checkout', default='.')
parser.add_argument(
    '--out', type=str, help='path to directory that will contain files', default='./out')
parser.add_argument(
    '--tmp', type=str, help='path to directory that will contain temporary files', default=None)
parser.add_argument(
    '--filter', type=str, help='comma-separated list of components that should be built', default='svaluer,jtl'
)

args = parser.parse_args()
# TODO: not create temp dir if one is already provided
with tempfile.TemporaryDirectory() as tmp:
    if args.tmp is None:
        args.tmp = tmp
    main(args)
