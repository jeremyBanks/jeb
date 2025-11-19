#!/bin/bash
jeb split-lines < in.txt > out.txt
echo $? > status.txt
