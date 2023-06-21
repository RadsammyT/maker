#include <iostream>
#include "parsing.hpp"
#include <vector>
#include <string>
#include <functional>
#include <optional>


int main(int argc, char* argv[]) {
	std::vector<std::string_view> args(argv+1, argv + argc);
	std::vector<std::string> inputFiles;
	flags flag = {
		.outputDir = std::nullopt,
		.help = false,
	};
	ParseArguments(args, inputFiles, flag);

	if(flag.help) {
		printf("maker, a wrapper for single-source compiling.\n\n"
				"Usage: maker [options] [file(s)]\n"
				"-o - Output Directory, where compiled source files go.\n"
				"-h / --help - Print this help screen.\n\n");
		return 0;
	}

	printf("Parsed arguments\n");
	for(auto i: args) {
		std::cout << i << " ";
		printf("\n");
	}
	printf("Parsed input files\n");
	for(auto i: inputFiles) {
		std::cout << i << " ";
		printf("\n");
	}
	printf("Parsed output file\n");
	if(flag.outputDir)
		printf("%s\n", flag.outputDir->get().c_str());
	else 
		printf("None.\n");


	CompileInput(inputFiles, flag);

	return 0;
}
