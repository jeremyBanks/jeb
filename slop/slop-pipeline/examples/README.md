# jeb Examples

This directory contains examples demonstrating the capabilities of jeb's pipeline-based architecture.

## Running the Examples

Each example is a shell script that can be run directly:

```bash
chmod +x examples/*.sh
./examples/01-basic-json.sh
```

Or run individual commands from the examples:

```bash
echo '{"hello":"world"}' | jeb parse-json to-json
```

## Example Categories

### 01-basic-json.sh
- Parsing and pretty-printing JSON
- Processing JSON from files
- Working with JSON Lines format
- Splitting and joining JSON arrays

### 02-text-processing.sh
- Splitting text by lines
- Collapsing whitespace
- Selecting first/last N items
- Joining text with different separators

### 03-encoding.sh
- Z85 encoding (ASCII-85 variant)
- JEB85 encoding (text-preserving)
- Encoding binary files
- Self-inspection (reading the jeb binary)

### 04-pipeline-architecture.sh
- Pipeline visualization (dry-run mode)
- Detailed pipeline view
- Understanding implicit commands
- Quiet mode for production use
- Chaining multiple files

### 05-advanced.sh
- Filtering empty items
- Combining transformations
- Working with structured data
- Exit status and error handling
- Complex multi-stage pipelines

## Key Concepts

### Pipeline Architecture
jeb builds a directed acyclic graph (DAG) of operations:
- Commands are connected using stack-based resolution
- Outputs from previous commands feed into subsequent commands
- Implicit `stdin` and `stdout` are added automatically

### Data Types
Three types flow through pipelines:
- **Text**: Unparsed UTF-8 strings
- **Bytes**: Raw binary data
- **Structured**: Parsed data (extended JSON model)

### Pipeline Visualization
By default, jeb shows the pipeline before execution:
- Implicit commands are shown in RED
- Explicit commands are shown in YELLOW
- Use `--dry-run` to see the pipeline without executing
- Use `-q` or `--quiet` to suppress visualization

### Exit Status
jeb uses exit codes to indicate where errors occurred:
- `0`: Success, no warnings
- `63-96`: Error at node index (63 + node_index)

## Common Patterns

### JSON Processing
```bash
# Parse and transform
jeb parse-json sort to-json

# Extract array elements
jeb parse-json split-array to-json

# Combine multiple JSON files
jeb file1.json file2.json chain parse-json to-json
```

### Text Manipulation
```bash
# Get first 10 lines
jeb split-lines first-10 join-lines

# Collapse whitespace
jeb collapse

# Sort lines
jeb split-lines sort join-lines
```

### Binary Encoding
```bash
# Encode with Z85
jeb encode-z85

# Encode with JEB85 (preserves text)
jeb encode-jeb85
```

## Advanced Usage

### Custom Pipelines
```bash
# Complex transformation
jeb ./data.json parse-json split-array first-100 sort join-array to-json

# Multi-file processing
jeb file*.json chain parse-json to-json

# Filter and transform
jeb split-lines filter collapse join-space
```

### Debugging
```bash
# Visualize the pipeline
jeb --dry-run your commands here

# See detailed node information
jeb --visualize-detailed your commands here

# Check exit status
jeb parse-json to-json; echo $?
```

## Tips

1. **Start Simple**: Begin with basic pipelines and add complexity gradually
2. **Use --dry-run**: Visualize pipelines before running them on large data
3. **Check Exit Codes**: Use exit status to detect errors in scripts
4. **Combine Operations**: jeb shines when chaining multiple transformations
5. **File Paths**: Use `./path` or `/path` to read files directly
