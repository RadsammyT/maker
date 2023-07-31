#include <algorithm>
#include <iostream>
#include <vector>
#include <string>
#include <stdexcept>
#include <filesystem>
#include <fstream>
#include <map>
#ifdef _WIN32
#define OUT_SUFFIX ".exe"
#else
#define OUT_SUFFIX ""
#endif
namespace fs = std::filesystem;
struct flags {
	std::string outputDir;
	bool help;
	bool breakOnNotZero;
};
// associate extension with configuration consisting of:
// compiler, format of execution
struct MakerLangConfig {
	std::string format;
};

namespace FILE_TYPE {
	enum FILE_TYPE { // add-lang
		C,
		CPP,
		RS, // Rust source file extension
		ZIG,
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

std::vector<std::string> tokenize(std::string in, char del) {
	std::string buf;
	std::vector<std::string> ret;
	std::stringstream instr(in);
	while(std::getline(instr, buf, del)) 
		ret.push_back(buf);
	return ret;
}
// convert format spec to include actual input file name and output dir
std::string ParseFormat(std::string inFile, flags flag, std::map<std::string, MakerLangConfig> cfgs) {
	std::string ext = inFile.substr(inFile.find_last_of('.'), inFile.size());
	std::string fmt = cfgs.at(ext).format;
	std::string outFile = inFile;
	outFile.replace(outFile.find(ext), sizeof(ext.c_str()), OUT_SUFFIX);
	std::string dir = string_format("%s/%s", flag.outputDir.c_str(), outFile.c_str());
	fmt.replace(fmt.find("%file%"), sizeof("%file%")-1, inFile);
	fmt.replace(fmt.find("%output%"), sizeof("%output%")-1, dir);
	return fmt;
}

/**
 * REWRITE TODO:
 * format should be like this:
 * |extension .cpp
 * |format g++ %file% -o %output%
 * |push
 *
 * why get the compiler alone? should that be included in just the format?
 * a format like this only needs files and output as required labels
 */
int GetMakerConfig(std::string input,
		flags flag,
		std::map<std::string, MakerLangConfig>& configs) {

	fs::path config;
	if(!fs::exists(input)) {
		printf("Unable to find file: %s \n", input.c_str());
		return 404;
	}
	if(!fs::exists((fs::absolute(input).parent_path() / ".maker"))) {
		printf("Unable to find configs for %s, by getting: ", input.c_str());
		std::cout << (fs::absolute(input).parent_path() / ".maker") << "\n";
	}

	std::ifstream dotMaker((fs::absolute(input).parent_path() / ".maker").string());
	std::string line;
	std::string format;
	std::vector<std::string> tokens ;
	// push config per extension
	std::vector<std::string> extensions;
	while(std::getline(dotMaker, line)) {
		if(!line.ends_with(" ")) {
			line += " ";
		}
		tokens = tokenize(line, ' ');
		if(tokens[0] == "extension") {
			tokens.erase(tokens.begin());
			tokens.swap(extensions);
			continue;
		}
		if(tokens[0] == "format") {
			tokens.erase(tokens.begin());
			for(auto i: tokens) {
				format += i;
				format += " ";
			}
			continue;
		}
		if(tokens[0] == "push") {
			for(auto i: extensions) {
				configs[i] = MakerLangConfig {
					.format = format
				};
			}
			format.clear();
			extensions.clear();
			continue;
		}
	}
	configs.erase("");
	return 0;
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
	
		if(str.starts_with("-")) {
			printf("Invalid argument: %s\n", strc.c_str());
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

		printf("UH OH!!! Passed a string! String is %s, with mode being %d.\n", 
				std::string(str.begin(), str.end()).c_str(), 
				cmode);
		return false;
	}
	return true;
}

int CompileInput(std::vector<std::string> inputFiles, flags flag) {
	int retCode = 0;
	std::string outFile;
	std::map<std::string, MakerLangConfig> makerCfg;
	if(flag.outputDir == "__MAKER_NULL") {
		flag.outputDir = "bin";
	}
	for(auto file: inputFiles) {
		GetMakerConfig(file, flag, makerCfg);	
		// Create output dir if one does not exist
		if(!fs::exists(	fs::absolute(file).parent_path() / flag.outputDir)) {
			fs::create_directory(fs::absolute(file).parent_path() / flag.outputDir);
		}

		printf("---%s---\n", file.c_str());
		std::string fmt = ParseFormat(file, flag, makerCfg).c_str();
		retCode = system(fmt.c_str());
		if(retCode != 0 ) {
			printf("Got return code %d for %s\n", retCode, file.c_str());
			if(flag.breakOnNotZero)
				return 1;
		}
	}
	return 0;
}
