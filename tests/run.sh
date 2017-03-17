#!/usr/bin/env bash

eval "$*"
CODE="$?"
if [ "$CODE" -eq "99" ]
then
    echo "Exit code: $CODE, test succeed."
    true
else
    echo "Exit code: $CODE, test failed."
    false
fi
