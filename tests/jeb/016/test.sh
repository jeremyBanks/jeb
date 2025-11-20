#!/bin/bash
jeb split-64k first split-64b encode-z85 join-lines < in.bin > out.txt
echo $? > status.txt
