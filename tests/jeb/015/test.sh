#!/bin/bash
jeb decode-z85 < in.txt > out.txt
echo $? > status.txt
