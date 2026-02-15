#!/bin/bash

BUILD_DIR="/mnt/c/Users/master/Documents/tilemap-sandbox/"
CMD_EXE="/mnt/c/Windows/System32/cmd.exe"

# copy files
rsync -C --filter=":- .gitignore" -acvz --delete . $BUILD_DIR

# build
cd $BUILD_DIR
$CMD_EXE /c "cargo build --release"
$CMD_EXE /c "copy target\release\native_main.dll native_main.dll"
