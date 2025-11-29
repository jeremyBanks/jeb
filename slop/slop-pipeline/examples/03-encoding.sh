#!/bin/bash
# Example 3: Binary encoding with Z85 and JEB85

echo "=== Example 1: Encode text with Z85 ==="
echo 'Hello, World!' | jeb encode-z85
echo

echo "=== Example 2: Encode binary data with JEB85 ==="
echo 'Binary data: \x00\x01\x02\x03' | jeb encode-jeb85
echo

echo "=== Example 3: Read binary file and encode ==="
# Create a small binary file
printf '\x00\x01\x02\x03\x04\x05\x06\x07' > /tmp/binary.dat
jeb /tmp/binary.dat encode-z85
echo

echo "=== Example 4: Encode the jeb binary itself (first 100 bytes) ==="
jeb self first-100
echo
