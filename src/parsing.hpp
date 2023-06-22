#include <iostream>
#include <vector>
#include <string>
#include <functional>
#include <optional>
#include <memory>
#include <stdexcept>

#define OP_REFW_STR std::optional<std::reference_wrapper<std::string>>
#pragma once
struct flags {
	std::string outputDir;
	bool help;
	bool breakOnNotZero;
};

namespace FILE_TYPE {
	enum FILE_TYPE {
		C,
		CPP,
		RS // Rust source file extension
	};
}
// yoinked from https://stackoverflow.com/questions/2342162/stdstring-formatting-like-sprintf
// answered by iFreilicht, thanks!
template<typename ... Args>
std::string string_format( const std::string& format, Args ... args )
{
    int size_s = std::snprintf( nullptr, 0, format.c_str(), args ... ) + 1; // Extra space for '\0'
    if( size_s <= 0 ){ throw std::runtime_error( "Error during formatting." ); }
    auto size = static_cast<size_t>( size_s );
    std::unique_ptr<char[]> buf( new char[ size ] );
    std::snprintf( buf.get(), size, format.c_str(), args ... );
    return std::string( buf.get(), buf.get() + size - 1 ); // We don't want the '\0' inside
}
int GetFileExtension(std::string_view in) {
	if(in.ends_with(".c")) {
		return FILE_TYPE::C;
	}

	if(in.ends_with(".cpp")) {
		return FILE_TYPE::CPP;
	}

	if(in.ends_with(".rs")) {
		return FILE_TYPE::RS;
	}

	return -1;
}

int ParseArguments(std::vector<std::string_view>& args, std::vector<std::string>& input,
					flags& flag) {
	enum mode {
		MODE_INPUT = 0, // Read each string as if they are input files
		MODE_OUTPUT, // Read as if the string is output directory
	};
	int cmode = MODE_INPUT;
	/*
	 * Order of processing:
	 * Do the flags first, otherwise we risk confusing
	 * a flag argument with an input/output argument
	 */

	for(auto str: args) {
		std::string strc(str.begin(), str.end());
		if(str == "-o") {
			cmode = MODE_OUTPUT;
			continue;
		}

		if(str == "-b") {
			flag.breakOnNotZero = true;
			continue;
		}

		if(str == "-h" || str == "--help") {
			flag.help = true;
			continue;
		}

		if(cmode == MODE_INPUT) {
			input.push_back(std::string(str.begin(), str.end()));
			continue;
		}

		if(cmode == MODE_OUTPUT) {
			flag.outputDir = strc;
			cmode = MODE_INPUT;
			continue;
		}

		printf("UH OH!!! Passed a string! String is %s, with mode being %d.", 
				std::string(str.begin(), str.end()).c_str(), 
				cmode);
		return false;
	}
	return true;
}

int CompileInput(std::vector<std::string> inputFiles, flags flag) {
	int retCode = 0;
	std::string outFile;
	for(auto file: inputFiles) {
		outFile = file;
		switch(GetFileExtension(std::string_view(file))) {

			case FILE_TYPE::CPP:
				outFile.replace(file.find(".cpp"), sizeof(".cpp"), ".exe");
				retCode = system(string_format("g++ %s -o %s %s",
							file.c_str(),
							 flag.outputDir != "__MAKER_NULL"?
							 string_format("%s/%s", flag.outputDir.c_str(), outFile.c_str()).c_str()
							: string_format("bin/%s", outFile.c_str()).c_str(),
							" "
							).c_str());
				break;

			case FILE_TYPE::C:

				break;

			case FILE_TYPE::RS:
					
				break;

			default:
				printf("Unknown/unsupported file extension for %s\n", file.c_str());
				break;
		}
		if(retCode != 0 ) {
			printf("Got return code %d for %s\n", retCode, file.c_str());
			if(flag.breakOnNotZero)
				return 1;
		}
	}
	return 0;
}
