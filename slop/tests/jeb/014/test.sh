#!/bin/bash
jeb split-lines join-lines < in.txt > out.txt
echo $? > status.txt
