#!/bin/bash

LANGUAGE=$1
CODE=$2

execute_code() {
  cmd=$1
  code=$2

  mkdir -p /home/executor/sandbox
  cp $(which $cmd) /home/executor/sandbox/
  echo "$code" > /home/executor/sandbox/code
  /home/executor/sandbox/$(basename $cmd) /home/executor/sandbox/code 2>&1
  EXIT_CODE=$?
  if [ $EXIT_CODE -ne 0 ]; then
    echo "EXECUTOR_ERROR"
    exit 1
  fi
}

compile_and_execute_rust() {
  code=$1

  echo "$code" > /home/executor/sandbox/temp.rs
  COMPILE_RESULT=$(rustc /home/executor/sandbox/temp.rs -o /home/executor/sandbox/temp 2>&1)
  COMPILE_EXIT_CODE=$?
  if [ $COMPILE_EXIT_CODE -ne 0 ]; then
    echo "$COMPILE_RESULT"
    echo "EXECUTOR_ERROR"
    exit 1
  fi
  EXEC_RESULT=$(/home/executor/sandbox/temp 2>&1)
  EXEC_EXIT_CODE=$?
  echo "$EXEC_RESULT"
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
