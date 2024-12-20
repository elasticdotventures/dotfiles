#!/bin/bash

# https://github.com/chvolkmann/code-connect

# Use this script through an alias
#   alias code="/path/to/code.sh"

local_code_executable="$(which code 2>/dev/null)"
if test -n "$local_code_executable"; then
    # code is in the PATH, use that binary instead of the code-connect
    $local_code_executable $@
else
    # code not locally installed, use code-connect to use vscode magic.
    ~/.dotfiles/vscode.🆚/.code-connect/bin/code_connect.py $@
fi
