#!/bin/bash
jeb split-64k encode-z85 join-lines < in.bin > out.txt
echo $? > status.txt
