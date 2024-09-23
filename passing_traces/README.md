# Passing Traces

This directory contains traces for which the model produces the same output
as the real chip. When you update the model to pass some new behavior, you
can move the trace from the failing_traces folder here.

## Trace Organization
Traces contain a "key section", which is between the signed data markers
- "===BEGIN SIGNED DATA===" and
- "===END SIGNED DATA==="
Immediately after the key section and the markers, is a line containing 
a signature.

When adding files to this repository, make sure they contain:
- both signed data markers, 
- the key section (within the markers)
- the following signature line.

