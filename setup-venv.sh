#! /bin/sh
# setup virtual env.

rm -rf .venv
python3 -m venv .venv
source .venv/bin/activate

python3 -m pip install numpy
python3 -m pip install pillow
