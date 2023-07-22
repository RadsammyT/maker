# Overview

Maker is a simple wrapper designed for single-source compilation. This software is particularlly 
useful for simple compilation of test programs that are contained in one single source file.

# Behaviour 

Maker will accept one or more source files, and will have their source files compiled into 
their respective binaries. By default they will be put into the `bin` directory for organization.

# Support

Maker supports Rust, Zig, C (GCC only) and C++ (G++ only) at the moment. 

# Configuration

You can configure compilation commands for a specific language through a `.maker` file 
located alongside your chosen source files.

To setup a configuration for a specific language (in this case C):

```
c= -g -ggdb -Og
```

Note that the `c=` (or any other config line) must be exact before writing the rest of that
languages config. Typically, specifying a language means that languages source file extension
and `=`.

