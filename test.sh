#!/usr/bin/bash

wasm-pack build --target web
python -m http.server
