#!/usr/bin/env bash

set -eou pipefail

readArg()
{
  local srcPath=data
  if [[ -z $1 ]]
  then
    ls $srcPath/*.asm
  else
    if [[ $1 =~ '^[0-9]+$' ]] 
    then
      ls $srcPath/*$1*.asm
    else
      echo $1
    fi
  fi
}

readSrc()
{
  cat $1 | grep -vE ';|bits 16|^$'
}

readObj()
{
  ./sim8086.bin -objPath $1
}

diffSourceWithParsed()
{
  local srcAsm="$(readSrc $1)"
  local objAsm="$(readObj $2)"
  local red='\033[1;31m'
  local green='\033[1;32m'
  local nocolor='\033[0m'

  local output=$(diff -u <(echo "$srcAsm") <(echo "$objAsm") | delta --color-only --side-by-side)
  if [[ -z $output ]]
  then
    echo -e "${green}Success${nocolor} for $1"
  else
    echo -e "${red}Fail${nocolor} for $1" >&2
    echo -e "$output"
    exit 1
  fi
}

run ()
{
  for srcPath in ${@}
  do
    if [[ $srcPath != *.asm ]]; then
      echo "$srcPath is not a source file"
      exit 1
    fi
    local objPath="${srcPath%.*}"
    diffSourceWithParsed $srcPath $objPath
  done
}

args=$(readArg ${1:-""})
run $args
