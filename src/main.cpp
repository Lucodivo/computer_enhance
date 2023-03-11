#include "main.h"

#pragma optimize("", off)
int main(int argCount, char** args)
{
	if(argCount != 2) {
		printf("You have not provided an assembly file. Terminating...");
		return -1;
	}

	char* programSystemPath = args[0];
	char* asmFilePath = args[1];
	read8086Mnemonic(asmFilePath);
}