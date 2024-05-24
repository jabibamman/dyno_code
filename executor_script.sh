#!/bin/sh

LANGUAGE=$1
CODE=$2

case $LANGUAGE in
  "python")
    echo "$CODE" | python3
    ;;
  "lua")
    echo "$CODE" | lua
    ;;
  "rust")
    echo "$CODE" > temp.rs
    rustc temp.rs && ./temp
    ;;
  *)
    echo "Unsupported language: $LANGUAGE"
    exit 1
    ;;
esac
