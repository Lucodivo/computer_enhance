#include "main.h"

struct {
	const char* exec = "-exec";
	const char* dump = "-dump";
} ConsoleArgs;

int main(int argCount, char** args)
{
	u32 filePathArgIndex = argCount - 1;
	char* programSystemPath = args[0];
	char* asmFilePath = args[filePathArgIndex];
	DecodeOptions options = {};

	for(int i = 1; i < (argCount - 1); i++) {
		if(strcmp(args[i], ConsoleArgs.exec) == 0) {
			options.execute = true;
		} else if(strcmp(args[i], ConsoleArgs.dump) == 0) {
			options.dump = true;
		} else {
			printf("Error: Bad arguments.");
			printf("Program usage: program.exe {-exec/-dump} {binary_asm_file}");
			return -1;
		}
	}

	decode8086Binary(asmFilePath, options);
}