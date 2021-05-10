#!/usr/bin/env python3

import argparse
import tempfile
import os
import os.path
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
        install_dir = os.path.realpath(args.out)
        print("Configuring JTL")
        subprocess.run(["cmake", "-S", f"{args.source}/jtl", "-B", f"{args.tmp}/cmake", f"-DCMAKE_INSTALL_PREFIX={install_dir}"], check=True)
        print("Building JTL")
        subprocess.run(["cmake", "--build", f"{args.tmp}/cmake"], check=True)
        print(f"Installing JTL to {install_dir}")
        subprocess.run(["cmake", "--install", f"{args.tmp}/cmake"], check=True)

DESCRIPTION = '''
Script that creates PPS problem build environment.
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
