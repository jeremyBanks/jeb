#!/bin/bash
jeb split-64k encode-z85 join < in.bin > out.txt
echo $? > status.txt
