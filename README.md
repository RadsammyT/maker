# Overview

Maker is a simple wrapper designed for single-source compilation. This software is particularlly 
useful for simple compilation of test programs that are contained in one single source file.

# Behaviour 

Maker will accept one or more source files, and will have their source files compiled into 
their respective binaries. By default they will be put into the `bin` directory for organization.

# Configuration

You can configure compilation commands for a specific language through a `.maker` file 
located alongside your chosen source files.

To setup a configuration for a specific language (in this case C):

```
extension .c
format gcc %file% -o %output%
push
```

The extension can take in multiple extensions: 

`extension .c .cpp .cc`

And will still have the same format specified for all of them.


