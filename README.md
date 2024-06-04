# Overview

Maker is a build system designed for single-source files. This software is particularly 
useful for coding playgrounds (or a collection of standalone single-source files meant to have
individual binaries).

# Behavior 

Maker will accept one or more source files, and will have their source files compiled into 
their respective binaries. By default they will be put into the `bin` directory for organization.

# Configuration

You can configure compilation commands for a specific language through a `maker` file 
located alongside your chosen source files (or if there is no `maker` file in your current directory, it will use `~/maker` instead).

Configuring the specific command used (in the use case of multiple compilers, etc.) is also
possible. Add `config CONFIG_WORD` before its respective attribute(s) to set that for
`CONFIG_WORD`,. To set the configuration, add `-c CONFIG_WORD` as the arguments to maker.

If the attribute is set without a preceding configuration, then that attribute will be the default
configuration when a `-c` argument isn't present.

Comments in your source files CAN have a use, specifically to add flags for that source file. 
Add `comment (ex: //MAKER:, #MAKER:, etc.)` to declare the prefix (ideally a comment). Any 
instances of that prefix of the source file will then add to the flags.

`all-comment` can also be used (in default config) to add flags for the source file, and this 
will apply to every configuration regardless.

To setup a configuration for a specific language (in this case C):

```
extension .c # .cpp .cxx .cc #and yes comments exist
	config gcc
        format gcc %file% -o %output%
        comment //GCC:
	end-config
	config tcc
        format tcc %file% -o %output%
        comment //TCC:
    end-config
	
	format cc %file% -o %output%
    comment //CC:
    all-comment //ALL:
end-extension
```

The extension can take in multiple extensions: 

`extension .c .cpp .cc`

And will still have the same format specified for all of them.



