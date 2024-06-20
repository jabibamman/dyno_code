#!/bin/bash

LANGUAGE=$1
CODE=$2
INPUT=$3

execute_code() {
  cmd=$1
  code=$2
  input=$3

  mkdir -p /home/executor/sandbox
  cp $(which $cmd) /home/executor/sandbox/
  echo "$input" > /home/executor/sandbox/input

  # Ajouter automatiquement le code pour lire l'input pour Python et Lua
  if [[ $cmd == "python3" ]]; then
    echo -e "with open('/home/executor/sandbox/input') as f:\n    input_data = f.read()\n$code" > /home/executor/sandbox/code
  elif [[ $cmd == "lua" ]]; then
    echo -e "local file = io.open('/home/executor/sandbox/input', 'r')\nlocal input_data = file:read('*a')\nfile:close()\n$code" > /home/executor/sandbox/code
  else
    echo "$code" > /home/executor/sandbox/code
  fi

  /home/executor/sandbox/$(basename $cmd) /home/executor/sandbox/code 2>&1

  EXIT_CODE=$?
  if [ $EXIT_CODE -ne 0 ]; then
    echo "EXECUTOR_ERROR"
    exit 1
  fi
}

compile_and_execute_rust() {
  code=$1
  input=$2

  echo "$code" > /home/executor/sandbox/temp.rs
  echo "$input" > /home/executor/sandbox/input

  COMPILE_RESULT=$(rustc /home/executor/sandbox/temp.rs -o /home/executor/sandbox/temp 2>&1)
  COMPILE_EXIT_CODE=$?
  if [ $COMPILE_EXIT_CODE -ne 0 ]; then
    echo "$COMPILE_RESULT"
    echo "EXECUTOR_ERROR"
    exit 1
  fi
  EXEC_RESULT=$(/home/executor/sandbox/temp /home/executor/sandbox/input 2>&1)
  EXEC_EXIT_CODE=$?
  echo "$EXEC_RESULT"
  if [ $EXEC_EXIT_CODE -ne 0 ]; then
    echo "EXECUTOR_ERROR"
    exit 1
  fi
}

case $LANGUAGE in
  "python")
    execute_code "python3" "$CODE" "$INPUT"
    ;;
  "lua")
    execute_code "lua" "$CODE" "$INPUT"
    ;;
  "rust")
    compile_and_execute_rust "$CODE" "$INPUT"
    ;;
  *)
    echo "Unsupported language EXECUTOR_ERROR"
    exit 1
    ;;
esac
