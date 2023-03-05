// 8086 16-bit instruction for mov is something like this
// [100010|D|W][MOD|REG|R/M]
// D - 1 bit - If 0 the register field is not destination, if 1 it is
// W - 1 bit - Is this 16-bits or not?nasm - h (stands for "wide")
// MOD - 2 bits - Mov can operate on registers and memory. Are the operands memory or registers?
// REG - 3 bits - Encodes a register
// R/M - 3 bits - Encodes a register or potentially memory

const char INSTR_OP_MOV = 0b100010;
const char D_REGISTER_IS_SOURCE = 0b0;
const char D_REGISTER_IS_DEST = 0b1;
const char W_OPERANDS_ARE_BYTES = 0b0;
const char W_OPERANDS_ARE_WORDS = 0b1;

struct Registers {
    union {
        struct{
            u16 ax;
            u16 cx;
            u16 dx;
            u16 bx;
            u16 sp;
            u16 bp;
            u16 si;
            u16 di;
        };
        u16 slot16[8];
    };
    u8* slot8(u8 regCode) {
        u8 highMask = 0b100;
        u8 regMask = 0b011;

        Assert(regCode >= 0b000 && regCode <= 0b111);

        return ((u8*)slot16[regCode & regMask]) + ((regCode & highMask) >> 2);
    }
};

const char* registerSlot16Name(u8 regCode) {
    const char* regNames[] {
        "ax","cx", "dx", "bx",
        "sp", "bp", "si", "di"
    };
    Assert(regCode >= 0b000 && regCode <= 0b111);
    return regNames[regCode];
}

const char* registerSlot8Name(u8 regCode) {
    const char* reg8Names[] {
        "al","cl", "dl", "bl",
        "ah","ch", "dh", "bh"
    };
    Assert(regCode >= 0b000 && regCode <= 0b111);
    return reg8Names[regCode];
}

const char* registerSlotName(u8 regCode, u8 w) {
    if(w == W_OPERANDS_ARE_WORDS) {
        return registerSlot16Name(regCode);
    } else {
        return registerSlot8Name(regCode);
    }
}

const char* opName(u8 opCode) {
    Assert(opCode >= 0b000000 && opCode <= 0b111111);

    switch(opCode) {
        case INSTR_OP_MOV:
            return "mov";
        default:
            return "unk";
    }
}

const char* wWidth(u8 w) {
    Assert(w >= 0b0 && w <= 0b1);

    switch(w) {
        case W_OPERANDS_ARE_BYTES:
            return "byte";
        case W_OPERANDS_ARE_WORDS:
            return "word";
        default:
            return "unk";
    }
}

const char* dRegister(u8 d) {
    Assert(d >= 0b0 && d <= 0b1);

    switch(d) {
        case D_REGISTER_IS_DEST:
            return "destination";
        case D_REGISTER_IS_SOURCE:
            return "source";
        default:
            return "unk";
    }
}


#define SINGLE_OP_FILE "listing_0037_single_register_mov"
#define MANY_OP_FILE "listing_0038_many_register_mov"

#define INPUT_ASM_FOLDER "data\\asm"
#define INPUT_FILE_NAME MANY_OP_FILE
#define INPUT_FILE_LOCATION INPUT_ASM_FOLDER"\\"INPUT_FILE_NAME
#define OUTPUT_FOLDER "output"
#define OUTPUT_FILE_LOCATION OUTPUT_FOLDER"\\"INPUT_FILE_NAME".asm"
#define LONGEST_OP "move ax, bx\n"
#define OUTPUT_FILE_HEADER "; Instruction decoding on the 8086 Homework by Connor Haskins\n\nbits 16\n\n"
void read8086Mnemonic() {
    FILE *fileptr;
    char *buffer;
    long fileLen;

    fileptr = fopen(INPUT_FILE_LOCATION, "rb");
    fseek(fileptr, 0, SEEK_END);
    fileLen = ftell(fileptr);
    Assert((fileLen % 2) == 0);
    rewind(fileptr);

    buffer = (char *)malloc(fileLen * sizeof(char));
    fread(buffer, fileLen, 1, fileptr);
    fclose(fileptr);

    Assert(makeDirectory(OUTPUT_FOLDER));

    fileptr = fopen(OUTPUT_FILE_LOCATION, "w");
    Assert(fileptr);
    fwrite(OUTPUT_FILE_HEADER, sizeof(OUTPUT_FILE_HEADER) - 1, 1, fileptr);
    printf(OUTPUT_FILE_HEADER);
    
    u32 opCount = fileLen / 2;
    char* fileText = (char *)malloc(sizeof(LONGEST_OP));
    for(u32 i = 0; i < opCount; i++) {
        u32 firstByteIndex = i * 2;
        u8 firstByte = buffer[firstByteIndex];
        
        u8 OP_MASK = 0b11111100;
        u8 opCode = firstByte >> 2;
        const char* op = opName(opCode);

        u8 D_MASK = 0b00000010;
        u8 d = (firstByte & D_MASK) >> 1;
        const char* registerDestination = dRegister(d);
        
        u8 W_MASK = 0b00000001;
        u8 w = (firstByte & W_MASK);
        const char* registerWidth = wWidth(w);

        u8 secondByte = buffer[firstByteIndex + 1];
        
        u8 MODE_MASK = 0b11000000;
        u8 mode = secondByte >> 6;
        // TODO: debug mode information

        u8 REG_MASK = 0b00111000;
        u8 reg = (secondByte & REG_MASK) >> 3;
        const char* registerName = registerSlotName(reg, w);
        
        u8 R_M_MASK = 0b00000111;
        u8 regMem = secondByte & R_M_MASK;
        const char* regMemName = registerSlotName(regMem, w);

        const char* srcName;
        const char* destName;
        if(d == D_REGISTER_IS_DEST) {
            destName = registerName;
            srcName = regMemName;
        } else {
            destName = regMemName;
            srcName = registerName;
        }

        const char* FORMAT_STR = "%s %s, %s\n";
        u32 opCharSize = sprintf(fileText, FORMAT_STR, op, destName, srcName);
        fwrite(fileText, opCharSize, 1, fileptr);
        printf(fileText);
    }
    free(fileText);
    fclose(fileptr);
    free(buffer);
}