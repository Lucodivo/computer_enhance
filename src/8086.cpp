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

#define SINGLE_OP_FILE "listing_0037_single_register_mov"
#define MANY_OP_FILE "listing_0038_many_register_mov"
#define MORE_OP_FILE "listing_0039_more_movs"
#define CHALLENGE_OP_FILE "listing_0040_challenge_movs"

#define INPUT_ASM_FOLDER "data\\asm"
#define INPUT_FILE_NAME MORE_OP_FILE
#define INPUT_FILE_LOCATION INPUT_ASM_FOLDER"\\"INPUT_FILE_NAME
#define OUTPUT_FOLDER "output"
#define OUTPUT_FILE_LOCATION OUTPUT_FOLDER"\\"INPUT_FILE_NAME".asm"
#define LONGEST_OP "move ax, bx\n"
#define OUTPUT_FILE_HEADER "; Instruction decoding on the 8086 Homework by Connor Haskins\n\nbits 16\n"

enum X86_OP {
    X86_OP_MOV_RM_TO_REG,
    X86_OP_MOV_IMM_TO_RM,
    X86_OP_MOV_IMM_TO_REG,
    X86_OP_MOV_MEM_TO_ACC,
    X86_OP_MOV_ACC_TO_MEM
};

enum X86_REG {
    X86_REG_AX, X86_REG_CX, X86_REG_DX, X86_REG_BX,
    X86_REG_SP, X86_REG_BP, X86_REG_SI, X86_REG_DI,

    X86_REG_AL, X86_REG_CL, X86_REG_DL, X86_REG_BL,
    X86_REG_AH, X86_REG_CH, X86_REG_DH, X86_REG_BH
};

enum DIRECTION {
    DIR_REG_IS_SRC,
    DIR_REG_IS_DST
};

enum WIDTH {
    WIDTH_BYTE,
    WIDTH_WORD
};

enum MODE_FLAGS {
    MODE_FLAGS_REG = 1 << 0,
    MODE_FLAGS_REG1_EFF_ADDR = 1 << 1,
    MODE_FLAGS_REG2_EFF_ADDR = 1 << 2,
    MODE_FLAGS_HAS_DISPLACE_8BIT = 1 << 3,
    MODE_FLAGS_HAS_DISPLACE_16BIT = 1 << 4
};

struct Mode {
    X86_REG reg1;
    X86_REG reg2;
    u8 operandFlags;
};

enum OPERAND_FLAGS {
    OPERAND_FLAGS_REG = 1 << 0,
    OPERAND_FLAGS_MEM_REG1 = 1 << 1,
    OPERAND_FLAGS_MEM_REG2 = 1 << 2,
    OPERAND_FLAGS_DISPLACEMENT_8BITS = 1 << 3,
    OPERAND_FLAGS_DISPLACEMENT_16BITS = 1 << 4,
    OPERAND_FLAGS_IMM_8BITS = 1 << 5,
    OPERAND_FLAGS_IMM_16BITS = 1 << 6
};

struct X86Operand {
    union {
        X86_REG reg;
        X86_REG reg1;
    };
    X86_REG reg2;
    union {
        u16 displacement;
        u16 immediate;
    };
    u8 flags;
};

struct DecodedOp {
    X86_OP op;
    X86Operand firstOperand;
    X86Operand secondOperand;
    u8* nextByte;
};

const char* registerName(X86_REG reg) {
    const char* regNames[] {
        "ax","cx", "dx", "bx",
        "sp", "bp", "si", "di",

        "al","cl", "dl", "bl",
        "ah","ch", "dh", "bh"
    };
    return regNames[(u32)reg];
}

X86_REG decodeRegByte(u8 regCode) {
    X86_REG regs[] {
        X86_REG_AL, X86_REG_CL, X86_REG_DL, X86_REG_BL,
        X86_REG_AH, X86_REG_CH, X86_REG_DH, X86_REG_BH
    };
    Assert(regCode >= 0b000 && regCode <= 0b111);
    return regs[regCode];
}

X86_REG decodeRegWord(u8 regCode) {
    X86_REG regs[] {
        X86_REG_AX, X86_REG_CX, X86_REG_DX, X86_REG_BX,
        X86_REG_SP, X86_REG_BP, X86_REG_SI, X86_REG_DI
    };
    Assert(regCode >= 0b000 && regCode <= 0b111);
    return regs[regCode];
}

X86_REG decodeReg(u8 reg, WIDTH width) {
    switch(width) {
        case WIDTH_BYTE:
            return decodeRegByte(reg);
        case WIDTH_WORD:
            return decodeRegWord(reg);
    }
    InvalidCodePath return X86_REG_AX; 
}

DIRECTION decodeDirection(u8 d) {
    DIRECTION directions[] = { DIR_REG_IS_SRC, DIR_REG_IS_DST };
    Assert(d >= 0b0 && d <= 0b1);
    return directions[d];
}

WIDTH decodeWidth(u8 w) {
    WIDTH widths[] = { WIDTH_BYTE, WIDTH_WORD };
    Assert(w >= 0b0 && w <= 0b1);
    return widths[w];
}

Mode decodeMode(u8 mod, u8 rm, WIDTH width) {
    Assert(mod >= 0b00 && mod <= 0b11);
    Assert(rm >= 0b000 && rm <= 0b111);

    Mode result{};

    // register to register
    if(mod == 0b11) {
        result.reg1 = decodeReg(rm, width);
        result.operandFlags = turnOnFlags(result.operandFlags, MODE_FLAGS_REG);
        return result;
    }

    // direct access special case
    if(rm == 0b110 && mod == 0b00) {
        result.operandFlags = MODE_FLAGS_HAS_DISPLACE_16BIT;
        return result;
    }

    // load effective memory address to register
    X86_REG effAddrRegs0to4[4][2] = {
        {X86_REG_BX, X86_REG_SI},
        {X86_REG_BX, X86_REG_DI},
        {X86_REG_BP, X86_REG_SI},
        {X86_REG_BP, X86_REG_DI}
    };
    X86_REG effAddrRegs5to7[4] { X86_REG_SI, X86_REG_DI, X86_REG_BP, X86_REG_BX };

    if(rm < 0b100) {
        result.reg1 = effAddrRegs0to4[rm][0];
        result.reg2 = effAddrRegs0to4[rm][1];
        result.operandFlags = turnOnFlags(result.operandFlags, MODE_FLAGS_REG1_EFF_ADDR | MODE_FLAGS_REG2_EFF_ADDR);
    } else {
        result.reg1 = effAddrRegs5to7[rm - 4];
        result.operandFlags = turnOnFlags(result.operandFlags, MODE_FLAGS_REG1_EFF_ADDR);
    }
    result.operandFlags = turnOnFlags(result.operandFlags, mod == 0b01 ? MODE_FLAGS_HAS_DISPLACE_8BIT : 0b0);
    result.operandFlags = turnOnFlags(result.operandFlags, mod == 0b10 ? MODE_FLAGS_HAS_DISPLACE_16BIT : 0b0);
    return result;
}


DecodedOp decodeOp(u8* bytes) {
    u8 byte1 = *bytes++;
    DecodedOp result{};

    if(byte1 & (1 << 7)) { // 1xxxxxxxx
        if(byte1 & (1 << 6)) {  // 11xxxxxx
            const u8 IMM_TO_REG_MEM_MASK = 0b11000110;
            if((byte1 & IMM_TO_REG_MEM_MASK) == IMM_TO_REG_MEM_MASK) { // 1100011w

                // imm to mem
                result.op = X86_OP_MOV_IMM_TO_RM;
                WIDTH opWidth = decodeWidth(byte1 & 0b1);
                u8 byte2 = *bytes++; // mod 000 r/m
                Mode mode = decodeMode(byte2 >> 6, byte2 & 0b111, opWidth);

                if(mode.operandFlags & MODE_FLAGS_REG1_EFF_ADDR) { // mem = [reg + ?]
                    result.firstOperand.reg1 = mode.reg1;
                    result.firstOperand.flags = turnOnFlags(result.firstOperand.flags, OPERAND_FLAGS_MEM_REG1);
                }

                if(mode.operandFlags & MODE_FLAGS_REG2_EFF_ADDR) { // mem = [reg1 + reg2 + ?]
                    result.firstOperand.reg2 = mode.reg2;
                    result.firstOperand.flags = turnOnFlags(result.firstOperand.flags, OPERAND_FLAGS_MEM_REG2);
                }

                if(mode.operandFlags & MODE_FLAGS_HAS_DISPLACE_8BIT) { // imm to [mem + 8-bit displacement]
                    u8 displacement = *bytes++;
                    result.firstOperand.displacement = displacement;
                    result.firstOperand.flags = turnOnFlags(result.firstOperand.flags, OPERAND_FLAGS_DISPLACEMENT_8BITS);
                } else if(mode.operandFlags & MODE_FLAGS_HAS_DISPLACE_16BIT) { // imm to [mem + 16-bit displacement]
                    u16 displacement = *(u16*)bytes; bytes += 2;
                    result.firstOperand.displacement = displacement;
                    result.firstOperand.flags = turnOnFlags(result.firstOperand.flags, OPERAND_FLAGS_DISPLACEMENT_16BITS);
                }

                switch(opWidth) {
                    case WIDTH_BYTE: {
                        u8 immediate = *bytes++;
                        result.secondOperand.immediate = immediate;
                        result.secondOperand.flags = turnOnFlags(result.secondOperand.flags, OPERAND_FLAGS_IMM_8BITS);
                        break;
                    }
                    case WIDTH_WORD: {
                        u16 immediate = *(u16*)bytes; bytes += 2;
                        result.secondOperand.immediate = immediate;
                        result.secondOperand.flags = turnOnFlags(result.secondOperand.flags, OPERAND_FLAGS_DISPLACEMENT_16BITS);
                        break;
                    }
                }

            }
        } else { // 10xxxxxx
            if(byte1 & (1 << 5)) { // 101xxxxx
                if(byte1 & (1 << 4)) { // 1011wreg

                    // imm to reg
                    result.op = X86_OP_MOV_IMM_TO_REG;
                    WIDTH opWidth = decodeWidth(((byte1 & 0b00001000) >> 3));
                    result.firstOperand.reg = decodeReg(byte1 & 0b111, opWidth);
                    result.firstOperand.flags = turnOnFlags(result.firstOperand.flags, OPERAND_FLAGS_REG);
                    if(opWidth == WIDTH_BYTE) {
                        u8 imm = *bytes++;
                        result.secondOperand.immediate = imm;
                        result.secondOperand.flags = turnOnFlags(result.secondOperand.flags, OPERAND_FLAGS_IMM_8BITS);
                    } else if(opWidth == WIDTH_WORD) {
                        u16 imm = *(u16*)bytes; bytes += 2;
                        result.secondOperand.immediate = imm;
                        result.secondOperand.flags = turnOnFlags(result.secondOperand.flags, OPERAND_FLAGS_IMM_16BITS);
                    }

                } else { // 1010xxxx
                    const u8 MEM_TO_ACC_MASK = 0b10100000;
                    const u8 ACC_TO_MEM_MASK = 0b10100010;
                    if((byte1 & MEM_TO_ACC_MASK) == MEM_TO_ACC_MASK) { // 1010000w

                        // mem to acc
                        result.op = X86_OP_MOV_MEM_TO_ACC;
                        WIDTH opWidth = decodeWidth(byte1 & 0b1);
                        // TODO: challenge

                    } else if((byte1 & ACC_TO_MEM_MASK) == ACC_TO_MEM_MASK) { // 1010001w

                        // acc to mem
                        result.op = X86_OP_MOV_ACC_TO_MEM;
                        WIDTH opWidth = decodeWidth(byte1 & 0b1);
                        // TODO: challenge

                    }
                }
            } else { // 100xxxxx

                // reg/mem to/from reg
                const u8 REG_MEM_TO_FROM_REG_MASK = 0b10001000;
                if((byte1 & REG_MEM_TO_FROM_REG_MASK) == REG_MEM_TO_FROM_REG_MASK) { // 100010dw
                    result.op = X86_OP_MOV_RM_TO_REG;
                    DIRECTION dir = decodeDirection((byte1 & 0b00000010) >> 1);
                    WIDTH opWidth = decodeWidth(byte1 & 0b1);
                    u8 byte2 = *bytes++; // mod reg rm
                    X86_REG reg = decodeReg((byte2 & 0b00111000) >> 3, opWidth);
                    Mode mode = decodeMode(byte2 >> 6, byte2 & 0b111, opWidth);

                    X86Operand* regOperand = (dir == DIR_REG_IS_DST) ? &result.firstOperand : &result.secondOperand;
                    X86Operand* regMemOperand = (dir == DIR_REG_IS_DST) ? &result.secondOperand : &result.firstOperand;

                    regOperand->reg = reg;
                    regOperand->flags = turnOnFlags(regOperand->flags, OPERAND_FLAGS_REG);

                    if(mode.operandFlags & MODE_FLAGS_REG) { // reg to reg
                        regMemOperand->reg = mode.reg1;
                        regMemOperand->flags = turnOnFlags(regMemOperand->flags, OPERAND_FLAGS_REG);
                    } else { // mem to/from reg

                        if(mode.operandFlags & MODE_FLAGS_REG1_EFF_ADDR) {
                            regMemOperand->reg1 = mode.reg1;
                            regMemOperand->flags = turnOnFlags(regMemOperand->flags, OPERAND_FLAGS_MEM_REG1);
                        }

                        if(mode.operandFlags & MODE_FLAGS_REG2_EFF_ADDR) {
                            regMemOperand->reg2 = mode.reg2;
                            regMemOperand->flags = turnOnFlags(regMemOperand->flags, OPERAND_FLAGS_MEM_REG2);
                        }

                        if(mode.operandFlags & MODE_FLAGS_HAS_DISPLACE_8BIT) {
                            u8 displacement = *bytes++;
                            regMemOperand->displacement = displacement;
                            regMemOperand->flags = turnOnFlags(regMemOperand->flags, OPERAND_FLAGS_DISPLACEMENT_8BITS);
                        } else if(mode.operandFlags & MODE_FLAGS_HAS_DISPLACE_16BIT) {
                            u16 displacement = *(u16*)bytes; bytes += 2;
                            regMemOperand->displacement = displacement;
                            regMemOperand->flags = turnOnFlags(regMemOperand->flags, OPERAND_FLAGS_DISPLACEMENT_16BITS);
                        }
                    }
                }
            }
        }
    }

    result.nextByte = bytes;
    return result;
}

void read8086Mnemonic() {
    u8* inputFileBuffer;
    long fileLen;

    FILE* inputFile = fopen(INPUT_FILE_LOCATION, "rb");
    fseek(inputFile, 0, SEEK_END);
    fileLen = ftell(inputFile);
    rewind(inputFile);

    inputFileBuffer = (u8 *)malloc(fileLen * sizeof(u8));
    fread(inputFileBuffer, fileLen, 1, inputFile);
    fclose(inputFile);
    inputFile = nullptr;

    printf(OUTPUT_FILE_HEADER);
    
    u8* lastByte = inputFileBuffer + fileLen;
    u8* currentByte = inputFileBuffer;
    while(currentByte < lastByte) {
        DecodedOp op = decodeOp(currentByte); 
        currentByte = op.nextByte;

        printf("\n");
        printf("mov ");

        auto writeAndPrintOperand = [](X86Operand operand) {
            if(operand.flags & OPERAND_FLAGS_REG) { // dest is reg
                printf("%s", registerName(operand.reg));
            } else if(operand.flags & OPERAND_FLAGS_MEM_REG1) { // dest is mem
                printf("[%s", registerName(operand.reg1));
                if(operand.flags & OPERAND_FLAGS_MEM_REG2) {
                    printf(" + %s", registerName(operand.reg2));
                }
                if(operand.flags & OPERAND_FLAGS_DISPLACEMENT_8BITS) {
                    s8 displacement = *(s8*)&operand.displacement;
                    printf(" + %d", displacement);
                } else if(operand.flags & OPERAND_FLAGS_DISPLACEMENT_16BITS) {
                    s16 displacement = *(s16*)&operand.displacement;
                    printf(" + %d", displacement);
                }
                printf("]");
            } else if(operand.flags & OPERAND_FLAGS_IMM_8BITS) {
                s8 immediate = *(s8*)&operand.immediate;
                printf("%d", immediate);
            } else if(operand.flags & OPERAND_FLAGS_IMM_16BITS) {
                s16 immediate = *(s16*)&operand.immediate;
                printf("%d", immediate);
            }
        };

        writeAndPrintOperand(op.firstOperand);
        printf(", ");
        // TODO: write comma space to file
        writeAndPrintOperand(op.secondOperand);
    }
    free(inputFileBuffer);
}