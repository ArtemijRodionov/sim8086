#!/usr/bin/env bash

set -eou pipefail

all_files=$(ls computer_enhance/perfaware/part1/*.asm)
files=${1:-$all_files}

run ()
{
  for file in ${@}
  do
    if [[ $file != *.asm ]]; then
      echo "$file is not a source file"
      exit 1
    fi
    local filename="${file%.*}"
    echo $filename
  done
}

run $files
