#!/bin/sh

while read LINE; do
    expr ${LINE% *} + ${LINE#* } # do something with it here
done
