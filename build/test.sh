#!/bin/bash
while getopts ":v:" opt; do
  case $opt in
    v) VERSION="$OPTARG"
    ;;
    \?) echo "Invalid option -$OPTARG" >&2
    ;;
  esac
done

printf "Argument p_out is %s\n" "$VERSION"
