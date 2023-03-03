// 8086 16-bit instruction for mov is something like this
// [100010|D|W][MOD|REG|R/M]
// D - 1 bit - If 0 the register field is not destination, if 1 it is
// W - 1 bit - Is this 16-bits or not?nasm - h (stands for "wide")
// MOD - 2 bits - Mov can operate on registers and memory. Are the operands memory or registers?
// REG - 3 bits - Encodes a register
// R/M - 3 bits - Encodes a register or potentially memory

const char INSTR_OP_MOV = 0b100010;
const char D_REGISTER_IS_DEST = 0b0;
const char D_REGISTER_IS_SOURCE = 0b1;
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
        u16 slot[8];
    };
    u8* slot8(u8 regCode) {
        u8 highMask = 0b100;
        u8 regMask = 0b011;

        Assert(regCode >= 0b000 && regCode <= 0b111);

        return ((u8*)slot[regCode & regMask]) + ((regCode & highMask) >> 2);
    }

    const char* slotName(u8 regCode) {
        const char* regNames[] {
            "ax","cx", "dx", "bx",
            "sp", "bp", "si", "di"
        };
        Assert(regCode >= 0b000 && regCode <= 0b111);
        return regNames[regCode];
    }

    const char* slot8Name(u8 regCode) {
        const char* reg8Names[] {
            "al","cl", "dl", "bl",
            "ah","ch", "dh", "bh"
        };
        Assert(regCode >= 0b000 && regCode <= 0b111);
        return reg8Names[regCode];
    }
};

const char* opName(u8 opCode) {
    Assert(opCode >= 0b000000 && opCode <= 0b111111);

    switch(opCode) {
        case INSTR_OP_MOV:
            return "mov";
        default:
            return "unk";
    }
}


void read8086Mnemonic() {
    FILE *fileptr;
    char *buffer;
    long filelen;

    fileptr = fopen("data\\asm\\listing_0037_single_register_mov", "rb");
    fseek(fileptr, 0, SEEK_END);
    filelen = ftell(fileptr);
    rewind(fileptr);

    buffer = (char *)malloc(filelen * sizeof(char));
    fread(buffer, filelen, 1, fileptr);
    fclose(fileptr);

    u8 firstByte = buffer[0];
    u8 opCode = firstByte >> 2;
    const char* op = opName(opCode);

    u8 D_MASK = 0b00000010;
    u8 d = (firstByte & D_MASK) >> 1;
    
    u8 W_MASK = 0b00000001;
    u8 w = (firstByte & W_MASK);

    printf(
        "op code: %s\n"
        "D: %i\n"
        "W: %i\n", 
        op,
        d,
        w);

    free(buffer);
}