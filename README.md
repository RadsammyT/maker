# Overview

Maker is a build system designed for single-source files. This software is particularly 
useful for coding playgrounds (or a collection of standalone single-source files).

# Behavior 

Maker will accept one or more source files, and will have their source files compiled into 
their respective binaries. By default they will be put into the `bin` directory for organization.

# Configuration

You can configure compilation commands for a specific language through a `.maker` file 
located alongside your chosen source files (or if there is no `.maker` file in your current directory, it will use `~/.config/maker/.maker` instead).
(TODO: Change default dir to ~/.maker instead)

Configuring the specific command used (in the use case of multiple compilers, etc.) is also
possible. Add `config CONFIG_WORD` before its respective format to set that format for
`CONFIG_WORD`. To set the configuration, add `-c CONFIG_WORD` as the arguments to maker.

If the format is set without a preceding configuration, then that format will be the default
configuration when a `-c` argument isn't present.

To setup a configuration for a specific language (in this case C):

```
extension .c
	config gcc
	format gcc %file% -o %output%
	
	config tcc
	format tcc %file% -o %output%
	
	format cc %file% -o %output%
push
```

The extension can take in multiple extensions: 

`extension .c .cpp .cc`

And will still have the same format specified for all of them.



