#!/bin/bash

RELEASE_DIR=target/wasm32-unknown-unknown/release
PROJECT_NAME=bevy_jaipur
OUTPUT_DIR=dist

cargo build --release --target wasm32-unknown-unknown && \
    wasm-bindgen --out-dir ./$OUTPUT_DIR/ --target web $RELEASE_DIR/$PROJECT_NAME.wasm && \
    mkdir -p $OUTPUT_DIR/$RELEASE_DIR && \
    cp -R assets $OUTPUT_DIR && \
    cp index.html $OUTPUT_DIR && \
    cp $RELEASE_DIR/$PROJECT_NAME.d $RELEASE_DIR/$PROJECT_NAME.wasm $OUTPUT_DIR/$RELEASE_DIR && \
    cd $OUTPUT_DIR && \
    zip -r dist.zip index.html assets target $PROJECT_NAME*