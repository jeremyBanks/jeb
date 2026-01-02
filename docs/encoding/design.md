# JEB85 Design

## Goals

Opportunistic semitranslucent binary text encodings.

## Base

85

## Digits

Z85

## Other Characters

- `|`
- `_`
- `~`

## Hmm?

```
XXXX 8|tes ttest XXXXX
XXXX ~test ingXX XXXXX
XXXX _test edXXX XXXXX

That's cute but gives us no way to do more than ~85 in length.
If we up the minimum from 6 to 7 we have more options.

Or maybe we have all of the length between 0 and 7 as special prefixes we can
use, eh? That's 3 bits of data.
```
