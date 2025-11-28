#!/bin/bash
jeb decode-jeb85 < in.txt > out.txt
echo $? > status.txt
