#!/bin/bash
jeb encode-z85 split-lines join-lines < in.txt > out.txt
echo $? > status.txt
