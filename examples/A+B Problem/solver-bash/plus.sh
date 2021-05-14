#!/bin/sh

while read LINE; do
    echo $((${LINE% *} + ${LINE#* }))
done
