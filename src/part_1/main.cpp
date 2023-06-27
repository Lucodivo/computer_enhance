#include "main.h"

const char* usageHelpText = "Error: Bad arguments.\n"
							"Program usage: program.exe {-exec/-dump/-clocks} {binary_asm_file}";

struct {
	const char* exec = "-exec";
	const char* dump = "-dump";
	const char* clocks = "-clocks";
} ConsoleArgs;

int main(int argCount, char** args)
{

	// First argument is always the program path (e.g. "C:\Users\...\program.exe")
	char* programSystemPath = args[0];

	if(argCount < 2) {
		printf(usageHelpText);
		return -1;
	}

	DecodeOptions options = {};
	for(int i = 1; i < (argCount - 1); i++) {
		if(strcmp(args[i], ConsoleArgs.exec) == 0) {
			options.execute = true;
		} else if(strcmp(args[i], ConsoleArgs.dump) == 0) {
			options.dump = true;
		} else if(strcmp(args[i], ConsoleArgs.clocks) == 0) {
			options.clocks = true;
		} else {
			printf(usageHelpText);
			return -1;
		}
	}

	u32 filePathArgIndex = argCount - 1;
	char* asmFilePath = args[filePathArgIndex];

	if(!checkFileExists(asmFilePath)) {
		printf("Error: File %s does not exist.", asmFilePath);
		return -1;
	}

	decode8086Binary(asmFilePath, options);
}