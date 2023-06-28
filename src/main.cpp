#include <iostream>
#include "parsing.hpp"
#include <vector>
#include <string>
#include <functional>
#include <optional>
#include <stdio.h>

int main(int argc, char* argv[]) {
	std::vector<std::string_view> args(argv+1, argv + argc);
	std::vector<std::string> inputFiles;
	flags flag = {
		.outputDir = "__MAKER_NULL",
		.help = false,
		.breakOnNotZero = false,
	};
	ParseArguments(args, inputFiles, flag);

	if(flag.help || argc == 1) {
		printf("maker, a wrapper for single-source compiling.\n\n"
				"Usage: maker [options] [file(s)]\n"
				"-o - Output Directory, where compiled source files go.\n"
				"-b - Break when any compiler returns a code not zero (0)\n"
				"-h / --help - Print this help screen.\n\n"
				);
		return 0;
	}
#ifdef DEBUG
	printf("Parsed arguments\n");
	for(auto i: args) {
		std::cout << i << " ";
		printf("\n");
	}
	printf("\nParsed input files\n");
	for(auto i: inputFiles) {
		std::cout << i << " ";
		printf("\n");
	}
	printf("\nParsed output file\n");
	if(flag.outputDir != "__MAKER_NULL") {
		printf("%s\n", flag.outputDir.c_str());
		printf("End.\n");
	} else 
		printf("None.\n");
#endif


	CompileInput(inputFiles, flag);
	return 0;
}
