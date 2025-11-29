#!/bin/bash
jeb encode-jeb85 decode-jeb85 < in.bin > out.txt
echo $? > status.txt
