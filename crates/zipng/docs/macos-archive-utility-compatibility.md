# macOS Archive Utility Compatibility Issue

## Problem

macOS Archive Utility cannot open polyglot PNG+ZIP files created by zipng, displaying an "Unable to expand" or "invalid format" error. However, the command-line `unzip` tool can successfully extract the same files.

## Root Cause

**Archive Utility requires files to start with the ZIP "PK" signature**, while polyglot PNG+ZIP files start with the PNG signature (`89 50 4e 47`).

### Technical Details

Standard ZIP file structure:
```
[PK signature: 50 4b 03 04] ← Archive Utility expects this at the start
[Local file headers and data]
[Central directory]
[End of central directory (EOCD)]
```

Polyglot PNG+ZIP file structure:
```
[PNG signature: 89 50 4e 47] ← Archive Utility rejects this
[PNG chunks including IDAT with ZIP local file headers embedded]
[PNG IEND chunk]
[ZIP Central directory]
[ZIP EOCD]
```

### Why `unzip` Works

The command-line `unzip` tool:
- Scans backward from the end of the file looking for the EOCD signature (`50 4b 05 06`)
- Reads the central directory location from the EOCD
- Doesn't validate what's at the beginning of the file

This is technically compliant with the ZIP specification, which allows arbitrary data before the first local file header.

### Why Archive Utility Fails

Archive Utility performs stricter validation:
- Checks that the file starts with a PK signature
- Rejects files that don't match this pattern
- Does not scan for ZIP structures inside other file formats

## Confirmed Issue

This is a **known and documented limitation** of macOS Archive Utility when working with polyglot files:

- The [Polyglot-HTML-ZIP-PNG documentation](https://gildas-lormeau.github.io/Polyglot-HTML-ZIP-PNG/SUMMARY.html) specifically notes this issue
- Multiple forum discussions confirm Archive Utility's stricter validation compared to command-line tools
- The ZIP format specification does not require files to start with PK signatures, but Archive Utility enforces this

## Workaround

Use the command-line `unzip` tool instead:

```bash
unzip filename.png
```

This is the standard approach for extracting polyglot PNG+ZIP files on macOS and works reliably.

## Alternative Solutions (Not Recommended)

1. **Rename to .zip extension**: This doesn't actually fix the issue, as Archive Utility still checks the file header
2. **Create ZIP-first polyglots**: Would require restructuring the file format and might break PNG compatibility
3. **Strip PNG header before extraction**: Adds unnecessary complexity

## References

- [How to Create HTML/ZIP/PNG Polyglot Files](https://gildas-lormeau.github.io/Polyglot-HTML-ZIP-PNG/SUMMARY.html)
- [ZIP (file format) - Wikipedia](https://en.wikipedia.org/wiki/ZIP_(file_format))
- [Problems with zip files - Archive Utility | MacRumors Forums](https://forums.macrumors.com/threads/problems-with-zip-files-archive-utility.2346237/)

## Recommendation

**This is expected behavior, not a bug.** Users should be informed that:
- Polyglot PNG+ZIP files require command-line extraction on macOS
- The `unzip` command is the recommended tool
- Archive Utility compatibility would require compromising the polyglot nature of the files
