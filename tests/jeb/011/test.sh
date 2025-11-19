#!/bin/bash
jeb split-64k < in.bin > out.txt
echo $? > status.txt
