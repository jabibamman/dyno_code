#!/bin/bash

LANGUAGE=$1
CODE=$2

execute_code() {
  cmd=$1
  code=$2

  RESULT=$(echo "$code" | $cmd 2>&1)
  EXIT_CODE=$?
  echo "$RESULT"
  if [ $EXIT_CODE -ne 0 ]; then
    echo "EXECUTOR_ERROR"
    exit 1
  fi
}

compile_and_execute_rust() {
  code=$1

  echo "$code" > temp.rs
  COMPILE_RESULT=$(rustc temp.rs 2>&1)
  COMPILE_EXIT_CODE=$?
  if [ $COMPILE_EXIT_CODE -ne 0 ]; then
    echo "$COMPILE_RESULT"
    rm temp.rs
    echo "EXECUTOR_ERROR"
    exit 1
  fi
  EXEC_RESULT=$(./temp 2>&1)
  EXEC_EXIT_CODE=$?
  echo "$EXEC_RESULT"
  rm temp.rs temp
  if [ $EXEC_EXIT_CODE -ne 0 ]; then
    echo "EXECUTOR_ERROR"
    exit 1
  fi
}

case $LANGUAGE in
  "python")
    execute_code "python3" "$CODE"
    ;;
  "lua")
    execute_code "lua" "$CODE"
    ;;
  "rust")
    compile_and_execute_rust "$CODE"
    ;;
  *)
    echo "Unsupported language EXECUTOR_ERROR"
    exit 1
    ;;
esac
