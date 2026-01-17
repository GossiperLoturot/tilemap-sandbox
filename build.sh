#!/bin/bash

BUILD_DIR="/mnt/c/Users/master/Documents/tilemap-sandbox/"
CMD_EXE="/mnt/c/Windows/System32/cmd.exe"
MSYS2_EXE="\msys64\msys2_shell.cmd"

# copy files
rsync -C --filter=":- .gitignore" -acvz --delete . $BUILD_DIR

# build
cd $BUILD_DIR
$CMD_EXE /c "$MSYS2_EXE -defterm -here -no-start -ucrt64 -c 'cargo build --release'"
$CMD_EXE /c "$MSYS2_EXE -defterm -here -no-start -ucrt64 -c 'cp target/release/native_main.dll native_main.dll'"
