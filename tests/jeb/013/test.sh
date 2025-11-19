#!/bin/bash
jeb split-64k join < in.bin > out.txt
echo $? > status.txt
