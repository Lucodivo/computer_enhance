u8 setFlags(const u8 currentFlags, const u8 desiredFlags) {
    return currentFlags | desiredFlags;
}

u8 resetFlags(const u8 currentFlags, const u8 desiredFlags) {
    u8 inversedDesiredMask = ~desiredFlags;
    return currentFlags & inversedDesiredMask;
}

bool checkAllFlags(const u8 currentFlags, const u8 desiredFlags) {
    return (currentFlags & desiredFlags) == desiredFlags;
}

bool checkAnyFlags(const u8 currentFlags, const u8 desiredFlags) {
    return (currentFlags & desiredFlags) > 0;
}