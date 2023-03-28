#include "main.h"

struct {
	const char* exec = "-exec";
} ConsoleArgs;

int main(int argCount, char** args)
{
	if(argCount <= 1 || argCount > 3) {
		printf("Error: Bad arguments.");
		printf("Program usage: program.exe {-exec} {binary_asm_file}");
		return -1;
	}

	char* programSystemPath = args[0];
	bool execute = argCount == 2 ? false : strcmp(args[1], ConsoleArgs.exec) == 0;
	char* asmFilePath = argCount == 2 ? args[1] : args[2];

	decode8086Binary(asmFilePath, execute);
}