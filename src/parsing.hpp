#include <iostream>
#include <vector>
#include <string>
#include <functional>
#include <optional>
#include <memory>
#include <stdexcept>
#include <filesystem>
#include <fstream>
#include <map>

namespace fs = std::filesystem;
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

int GetMakerConfig(std::string input, std::map<int, std::string>& makerLangConfigs, flags flag) {
	fs::path config;
	if(!fs::exists(input)) {
		return 404;
	}
	if(!fs::exists((fs::absolute(input).parent_path() / ".maker"))) {
		std::cout << (fs::absolute(input).parent_path() / ".maker") << "\n";
		return 1;
	}

	std::ifstream dotMaker((fs::absolute(input).parent_path() / ".maker").string());
	int lineCount = 0;
	while(std::getline(dotMaker, input)) {
		lineCount++;
		if(input.starts_with("c=")) {
			input.erase(0, 2);
			makerLangConfigs[FILE_TYPE::C] = input;
			continue;
		}

		if(input.starts_with("cpp=")) {
			input.erase(0, 4);
			makerLangConfigs[FILE_TYPE::CPP] = input;
			continue;
		}

		if(input.starts_with("rs=")) {
			input.erase(0, 3);
			makerLangConfigs[FILE_TYPE::RS] = input;
			continue;
		}
		printf("Invalid config start on line %d!\n"
				"%s\n", lineCount, input.c_str());
		if(flag.breakOnNotZero) {
			return 1;
		}

	}
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
	std::map<int, std::string> makerCfg;
	for(auto file: inputFiles) {
		if(GetMakerConfig(file, makerCfg, flag) != 0) {
			return 1;
		}
		outFile = file;
		switch(GetFileExtension(std::string_view(file))) {

			case FILE_TYPE::CPP:
				outFile.replace(file.find(".cpp"), sizeof(".cpp"), ".exe");
				retCode = system(string_format("g++ %s -o %s %s",
							file.c_str(),
							 flag.outputDir != "__MAKER_NULL"?
							 string_format("%s/%s", flag.outputDir.c_str(), outFile.c_str()).c_str()
							: string_format("bin/%s", outFile.c_str()).c_str(),
							makerCfg[FILE_TYPE::CPP].c_str()
							).c_str());
				break;

			case FILE_TYPE::C:
				outFile.replace(file.find(".c"), sizeof(".c"), ".exe");
				retCode = system(string_format("gcc %s -o %s %s",
							file.c_str(),
							 flag.outputDir != "__MAKER_NULL"?
							 string_format("%s/%s", flag.outputDir.c_str(), outFile.c_str()).c_str()
							: string_format("bin/%s", outFile.c_str()).c_str(),
							makerCfg[FILE_TYPE::C].c_str()
							).c_str());
				break;

			case FILE_TYPE::RS:
				outFile.replace(file.find(".rs"), sizeof(".rs"), ".exe");
				retCode = system(string_format("rustc %s -o %s %s",
							file.c_str(),
							 flag.outputDir != "__MAKER_NULL"?
							 string_format("%s/%s", flag.outputDir.c_str(), outFile.c_str()).c_str()
							: string_format("bin/%s", outFile.c_str()).c_str(),
							makerCfg[FILE_TYPE::RS].c_str()
							).c_str());
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
