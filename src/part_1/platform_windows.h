#include <windows.h>

// simply returns true if directory exists after this call
bool makeDirectory(const char* dir) {
    BOOL mkDirResult = CreateDirectoryA(dir, NULL);
    return mkDirResult || (GetLastError() == ERROR_ALREADY_EXISTS);
}