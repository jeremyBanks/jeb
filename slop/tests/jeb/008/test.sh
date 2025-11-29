#!/bin/bash
jeb encode-z85 < in.bin > out.txt
echo $? > status.txt
