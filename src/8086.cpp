#define OUTPUT_FILE_HEADER "; Instruction decoding on the 8086 Homework by Connor Haskins\n\nbits 16\n"

enum X86_OP {
    X86_OP_UNDEFINED = 0,
    X86_OP_PARTIAL_OP,

    x86_OP_MOV__START,
    X86_OP_MOV_RM_TO_FROM_REG,
    X86_OP_MOV_IMM_TO_RM,
    X86_OP_MOV_IMM_TO_REG,
    X86_OP_MOV_MEM_TO_ACC,
    X86_OP_MOV_ACC_TO_MEM,
    X86_OP_MOV__END,

    X86_OP_ADD__START,
    X86_OP_ADD_RM_TO_FROM_REG,
    X86_OP_ADD_IMM_TO_RM,
    x86_OP_ADD_IMM_TO_ACC,
    X86_OP_ADD__END,

    X86_OP_ADC_IMM_TO_RM,
    X86_OP_SBB_IMM_FROM_RM,
    X86_OP_AND_IMM_WITH_RM,
    X86_OP_OR_IMM_WITH_RM,
    X86_OP_XOR_IMM_WITH_RM,

    X86_OP_SUB__START,
    X86_OP_SUB_RM_TO_FROM_REG,
    X86_OP_SUB_IMM_FROM_RM,
    X86_OP_SUB_IMM_FROM_ACC,
    X86_OP_SUB__END,

    X86_OP_CMP__START,
    X86_OP_CMP_RM_AND_REG,
    X86_OP_CMP_IMM_AND_RM,
    X86_OP_CMP_IMM_AND_ACC,
    X86_OP_CMP__END,

    X86_OP_JMP__START,
    X86_OP_JE_JZ,
    X86_OP_JL_JNGE,
    X86_OP_JLE_JNG,
    X86_OP_JB_JNAE,
    X86_OP_JBE_JNA,
    X86_OP_JP_JPE,
    X86_OP_JO,
    X86_OP_JS,
    X86_OP_JNE_JNZ,
    X86_OP_JNL_JGE,
    X86_OP_JNLE_JG,
    X86_OP_JNB_JAE,
    X86_OP_JNBE_JA,
    X86_OP_JNP_JPO,
    X86_OP_JNO,
    X86_OP_JNS,
    X86_OP_LOOP,
    X86_OP_LOOPZ_LOOPE,
    X86_OP_LOOPNZ_LOOPNE,
    X86_OP_JCXZ,
    X86_OP_JMP__END,
};

namespace X86_REG {
    enum Value { // NOTE: ORDER MATTERS
        AX, CX, DX, BX,
        SP, BP, SI, DI,

        AL, CL, DL, BL,
        AH, CH, DH, BH
    };
    const char* regName(Value val) {
        const char* regNames[] = { // NOTE: ORDER FOLLOWS ENUMS
                "ax","cx", "dx", "bx",
                "sp", "bp", "si", "di",

                "al","cl", "dl", "bl",
                "ah","ch", "dh", "bh"
        };
        return regNames[val];
    }
};

enum DIRECTION {
    DIR_REG_IS_SRC = 0,
    DIR_REG_IS_DST = 1
};

enum WIDTH {
    WIDTH_BYTE = 0,
    WIDTH_WORD = 1
};

enum MODE_FLAGS {
    MODE_FLAGS_REG = 1 << 0,
    MODE_FLAGS_REG1_EFF_ADDR = 1 << 1,
    MODE_FLAGS_REG2_EFF_ADDR = 1 << 2,
    MODE_FLAGS_HAS_DISPLACE_8BIT = 1 << 3,
    MODE_FLAGS_HAS_DISPLACE_16BIT = 1 << 4
};

struct Mode {
    union {
        X86_REG::Value reg;
        X86_REG::Value memReg1;
    };
    X86_REG::Value memReg2;
    u8 flags;
};

enum OPERAND_FLAGS {
    OPERAND_FLAGS_REG = 1 << 0,
    OPERAND_FLAGS_MEM_REG1 = 1 << 1,
    OPERAND_FLAGS_MEM_REG2 = 1 << 2,
    OPERAND_FLAGS_DISPLACEMENT = 1 << 3,
    OPERAND_FLAGS_IMM_8BITS = 1 << 4,
    OPERAND_FLAGS_IMM_16BITS = 1 << 5,
    OPERAND_FLAGS_NO_OPERAND = 1 << 6,
};

struct X86Operand {
    union {
        X86_REG::Value reg;
        X86_REG::Value memReg1;
    };
    X86_REG::Value memReg2;
    union {
        s16 displacement;
        s16 immediate;
    };
    u8 flags;
};

struct DecodedOp {
    X86_OP op;
    X86Operand operand1;
    X86Operand operand2;
    u8* nextByte;
};

// TODO: Special cases
namespace X86_OP_METADATA {
    enum Value : u32 {
        REG_IS_DST      = 1 << 0, // if 0, reg is src where applicable
        SIGN_EXT        = 1 << 1, // if 0, values are not sign extended
        SIZE_WORD       = 1 << 2, // if 0, values are 1 byte where applicable
        REG_BYTE1       = 1 << 3, // There's a reg code in byte 1
        MOD_RM          = 1 << 4, // Three's a MOD R/M in byte 2
        REG_BYTE2       = 1 << 5, // There's a reg code in byte 2
        PARTIAL_OP      = 1 << 6, // There's an addt'l op code in byte 2
        IMM             = 1 << 7, // Immediate data follows
        MEM             = 1 << 8, // Memory data follows
        ACC             = 1 << 9, // Op uses the accumulator (AX/AL)
        INC_IP_8BIT     = 1 << 10,// Signed increment to instruction pointer (jumps)
    };
};

struct OpMetadata { // NOTE: member order matters and is depended on
    X86_OP op;
    u32 flags;
    X86_OP* extOps;
};

X86_REG::Value decodeRegWord(u8 regCode) {
    Assert(regCode >= 0b000 && regCode <= 0b111);
    return X86_REG::Value(regCode + X86_REG::AX);
}

X86_REG::Value decodeRegByte(u8 regCode) {
    Assert(regCode >= 0b000 && regCode <= 0b111);
    return X86_REG::Value(regCode + X86_REG::AL);
}

X86_REG::Value decodeReg(u8 reg, WIDTH width) {
    return width == WIDTH_BYTE ? decodeRegByte(reg) : decodeRegWord(reg);
}

Mode decodeMode(u8 mod, u8 rm, WIDTH width) {
    Assert(mod >= 0b00 && mod <= 0b11);
    Assert(rm >= 0b000 && rm <= 0b111);

    Mode result{};

    // register to register
    if(mod == 0b11) {
        result.memReg1 = decodeReg(rm, width);
        result.flags = turnOnFlags(result.flags, MODE_FLAGS_REG);
        return result;
    }

    // direct access special case
    if(rm == 0b110 && mod == 0b00) {
        result.flags = MODE_FLAGS_HAS_DISPLACE_16BIT;
        return result;
    }

    // load effective memory address to register
    X86_REG::Value effAddrRegs0to4[4][2] = {
        {X86_REG::BX, X86_REG::SI},
        {X86_REG::BX, X86_REG::DI},
        {X86_REG::BP, X86_REG::SI},
        {X86_REG::BP, X86_REG::DI}
    };
    X86_REG::Value effAddrRegs5to7[4] { X86_REG::SI, X86_REG::DI, X86_REG::BP, X86_REG::BX };

    if(rm < 0b100) {
        result.memReg1 = effAddrRegs0to4[rm][0];
        result.memReg2 = effAddrRegs0to4[rm][1];
        result.flags = turnOnFlags(result.flags, MODE_FLAGS_REG1_EFF_ADDR | MODE_FLAGS_REG2_EFF_ADDR);
    } else {
        result.memReg1 = effAddrRegs5to7[rm - 4];
        result.flags = turnOnFlags(result.flags, MODE_FLAGS_REG1_EFF_ADDR);
    }
    result.flags = turnOnFlags(result.flags, mod == 0b01 ? MODE_FLAGS_HAS_DISPLACE_8BIT : 0b0);
    result.flags = turnOnFlags(result.flags, mod == 0b10 ? MODE_FLAGS_HAS_DISPLACE_16BIT : 0b0);
    return result;
}

DecodedOp decodeOp(u8* bytes) {

    DecodedOp result{};

    X86_OP extOps_1000_00xx[8] {
        X86_OP_ADD_IMM_TO_RM, // 000
        X86_OP_OR_IMM_WITH_RM, // 001
        X86_OP_ADC_IMM_TO_RM, // 010
        X86_OP_SBB_IMM_FROM_RM, // 011
        X86_OP_AND_IMM_WITH_RM, // 100
        X86_OP_SUB_IMM_FROM_RM, // 101
        X86_OP_XOR_IMM_WITH_RM, // 110
        X86_OP_CMP_IMM_AND_RM, // 111
    };

    using namespace X86_OP_METADATA;
    OpMetadata opMetadata[256] = {
        /* 0000 0000 */ {X86_OP_ADD_RM_TO_FROM_REG, MOD_RM | REG_BYTE2}, 
        /* 0000 0001 */ {X86_OP_ADD_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | SIZE_WORD}, 
        /* 0000 0010 */ {X86_OP_ADD_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | REG_IS_DST}, 
        /* 0000 0011 */ {X86_OP_ADD_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | REG_IS_DST | SIZE_WORD}, 
        /* 0000 0100 */ {x86_OP_ADD_IMM_TO_ACC, ACC | IMM | REG_IS_DST}, 
        /* 0000 0101 */ {x86_OP_ADD_IMM_TO_ACC, ACC | IMM | REG_IS_DST | SIZE_WORD}, 
        /* 0000 0110 */ {}, 
        /* 0000 0111 */ {}, 
        /* 0000 1000 */ {}, 
        /* 0000 1001 */ {}, 
        /* 0000 1010 */ {}, 
        /* 0000 1011 */ {}, 
        /* 0000 1100 */ {}, 
        /* 0000 1101 */ {}, 
        /* 0000 1110 */ {}, 
        /* 0000 1111 */ {}, 
        /* 0001 0000 */ {}, 
        /* 0001 0001 */ {}, 
        /* 0001 0010 */ {}, 
        /* 0001 0011 */ {}, 
        /* 0001 0100 */ {}, 
        /* 0001 0101 */ {}, 
        /* 0001 0110 */ {}, 
        /* 0001 0111 */ {}, 
        /* 0001 1000 */ {}, 
        /* 0001 1001 */ {}, 
        /* 0001 1010 */ {}, 
        /* 0001 1011 */ {}, 
        /* 0001 1100 */ {}, 
        /* 0001 1101 */ {}, 
        /* 0001 1110 */ {}, 
        /* 0001 1111 */ {}, 
        /* 0010 0000 */ {}, 
        /* 0010 0001 */ {}, 
        /* 0010 0010 */ {}, 
        /* 0010 0011 */ {}, 
        /* 0010 0100 */ {}, 
        /* 0010 0101 */ {}, 
        /* 0010 0110 */ {}, 
        /* 0010 0111 */ {}, 
        /* 0010 1000 */ {X86_OP_SUB_RM_TO_FROM_REG, MOD_RM | REG_BYTE2}, 
        /* 0010 1001 */ {X86_OP_SUB_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | SIZE_WORD}, 
        /* 0010 1010 */ {X86_OP_SUB_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | REG_IS_DST}, 
        /* 0010 1011 */ {X86_OP_SUB_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | REG_IS_DST | SIZE_WORD}, 
        /* 0010 1100 */ {X86_OP_SUB_IMM_FROM_ACC, ACC | IMM | REG_IS_DST}, 
        /* 0010 1101 */ {X86_OP_SUB_IMM_FROM_ACC, ACC | IMM | REG_IS_DST | SIZE_WORD}, 
        /* 0010 1110 */ {}, 
        /* 0010 1111 */ {}, 
        /* 0011 0000 */ {}, 
        /* 0011 0001 */ {}, 
        /* 0011 0010 */ {}, 
        /* 0011 0011 */ {}, 
        /* 0011 0100 */ {}, 
        /* 0011 0101 */ {}, 
        /* 0011 0110 */ {}, 
        /* 0011 0111 */ {}, 
        /* 0011 1000 */ {X86_OP_CMP_RM_AND_REG, MOD_RM | REG_BYTE2}, 
        /* 0011 1001 */ {X86_OP_CMP_RM_AND_REG, MOD_RM | REG_BYTE2 | SIZE_WORD}, 
        /* 0011 1010 */ {X86_OP_CMP_RM_AND_REG, MOD_RM | REG_BYTE2 | REG_IS_DST}, 
        /* 0011 1011 */ {X86_OP_CMP_RM_AND_REG, MOD_RM | REG_BYTE2 | REG_IS_DST | SIZE_WORD}, 
        /* 0011 1100 */ {X86_OP_CMP_IMM_AND_ACC, ACC | IMM | REG_IS_DST}, 
        /* 0011 1101 */ {X86_OP_CMP_IMM_AND_ACC, ACC | IMM | REG_IS_DST | SIZE_WORD},
        /* 0011 1110 */ {}, 
        /* 0011 1111 */ {}, 
        /* 0100 0000 */ {}, 
        /* 0100 0001 */ {}, 
        /* 0100 0010 */ {}, 
        /* 0100 0011 */ {}, 
        /* 0100 0100 */ {}, 
        /* 0100 0101 */ {}, 
        /* 0100 0110 */ {}, 
        /* 0100 0111 */ {}, 
        /* 0100 1000 */ {}, 
        /* 0100 1001 */ {}, 
        /* 0100 1010 */ {}, 
        /* 0100 1011 */ {}, 
        /* 0100 1100 */ {}, 
        /* 0100 1101 */ {}, 
        /* 0100 1110 */ {}, 
        /* 0100 1111 */ {}, 
        /* 0101 0000 */ {}, 
        /* 0101 0001 */ {}, 
        /* 0101 0010 */ {}, 
        /* 0101 0011 */ {}, 
        /* 0101 0100 */ {}, 
        /* 0101 0101 */ {}, 
        /* 0101 0110 */ {}, 
        /* 0101 0111 */ {}, 
        /* 0101 1000 */ {}, 
        /* 0101 1001 */ {}, 
        /* 0101 1010 */ {}, 
        /* 0101 1011 */ {}, 
        /* 0101 1100 */ {}, 
        /* 0101 1101 */ {}, 
        /* 0101 1110 */ {}, 
        /* 0101 1111 */ {}, 
        /* 0110 0000 */ {}, 
        /* 0110 0001 */ {}, 
        /* 0110 0010 */ {}, 
        /* 0110 0011 */ {}, 
        /* 0110 0100 */ {}, 
        /* 0110 0101 */ {}, 
        /* 0110 0110 */ {}, 
        /* 0110 0111 */ {}, 
        /* 0110 1000 */ {}, 
        /* 0110 1001 */ {}, 
        /* 0110 1010 */ {}, 
        /* 0110 1011 */ {}, 
        /* 0110 1100 */ {}, 
        /* 0110 1101 */ {}, 
        /* 0110 1110 */ {}, 
        /* 0110 1111 */ {}, 
        /* 0111 0000 */ {X86_OP_JO, INC_IP_8BIT}, 
        /* 0111 0001 */ {X86_OP_JNO, INC_IP_8BIT}, 
        /* 0111 0010 */ {X86_OP_JB_JNAE, INC_IP_8BIT}, 
        /* 0111 0011 */ {X86_OP_JNB_JAE, INC_IP_8BIT}, 
        /* 0111 0100 */ {X86_OP_JE_JZ, INC_IP_8BIT}, 
        /* 0111 0101 */ {X86_OP_JNE_JNZ, INC_IP_8BIT}, 
        /* 0111 0110 */ {X86_OP_JBE_JNA, INC_IP_8BIT}, 
        /* 0111 0111 */ {X86_OP_JNBE_JA, INC_IP_8BIT}, 
        /* 0111 1000 */ {X86_OP_JS, INC_IP_8BIT}, 
        /* 0111 1001 */ {X86_OP_JNS, INC_IP_8BIT}, 
        /* 0111 1010 */ {X86_OP_JP_JPE, INC_IP_8BIT}, 
        /* 0111 1011 */ {X86_OP_JNP_JPO, INC_IP_8BIT}, 
        /* 0111 1100 */ {X86_OP_JL_JNGE, INC_IP_8BIT}, 
        /* 0111 1101 */ {X86_OP_JNL_JGE, INC_IP_8BIT}, 
        /* 0111 1110 */ {X86_OP_JLE_JNG, INC_IP_8BIT}, 
        /* 0111 1111 */ {X86_OP_JNLE_JG, INC_IP_8BIT}, 
        /* 1000 0000 */ {X86_OP_PARTIAL_OP, MOD_RM | PARTIAL_OP | IMM, extOps_1000_00xx},
        /* 1000 0001 */ {X86_OP_PARTIAL_OP, MOD_RM | PARTIAL_OP | IMM | SIZE_WORD, extOps_1000_00xx},
        /* 1000 0010 */ {X86_OP_PARTIAL_OP, MOD_RM | PARTIAL_OP | IMM | SIGN_EXT, extOps_1000_00xx},
        /* 1000 0011 */ {X86_OP_PARTIAL_OP, MOD_RM | PARTIAL_OP | IMM | SIGN_EXT | SIZE_WORD, extOps_1000_00xx},
        /* 1000 0100 */ {}, 
        /* 1000 0101 */ {}, 
        /* 1000 0110 */ {}, 
        /* 1000 0111 */ {}, 
        /* 1000 1000 */ {X86_OP_MOV_RM_TO_FROM_REG, MOD_RM | REG_BYTE2}, 
        /* 1000 1001 */ {X86_OP_MOV_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | SIZE_WORD}, 
        /* 1000 1010 */ {X86_OP_MOV_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | REG_IS_DST}, 
        /* 1000 1011 */ {X86_OP_MOV_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | REG_IS_DST | SIZE_WORD}, 
        /* 1000 1100 */ {}, 
        /* 1000 1101 */ {}, 
        /* 1000 1110 */ {}, 
        /* 1000 1111 */ {}, 
        /* 1001 0000 */ {}, 
        /* 1001 0001 */ {}, 
        /* 1001 0010 */ {}, 
        /* 1001 0011 */ {}, 
        /* 1001 0100 */ {}, 
        /* 1001 0101 */ {}, 
        /* 1001 0110 */ {}, 
        /* 1001 0111 */ {}, 
        /* 1001 1000 */ {}, 
        /* 1001 1001 */ {}, 
        /* 1001 1010 */ {}, 
        /* 1001 1011 */ {}, 
        /* 1001 1100 */ {}, 
        /* 1001 1101 */ {}, 
        /* 1001 1110 */ {}, 
        /* 1001 1111 */ {}, 
        /* 1010 0000 */ {X86_OP_MOV_MEM_TO_ACC, ACC | MEM | REG_IS_DST}, 
        /* 1010 0001 */ {X86_OP_MOV_MEM_TO_ACC, ACC | MEM | REG_IS_DST | SIZE_WORD}, 
        /* 1010 0010 */ {X86_OP_MOV_ACC_TO_MEM, ACC | MEM}, 
        /* 1010 0011 */ {X86_OP_MOV_ACC_TO_MEM, ACC | MEM | SIZE_WORD},
        /* 1010 0100 */ {}, 
        /* 1010 0101 */ {}, 
        /* 1010 0110 */ {}, 
        /* 1010 0111 */ {}, 
        /* 1010 1000 */ {}, 
        /* 1010 1001 */ {}, 
        /* 1010 1010 */ {}, 
        /* 1010 1011 */ {}, 
        /* 1010 1100 */ {}, 
        /* 1010 1101 */ {}, 
        /* 1010 1110 */ {}, 
        /* 1010 1111 */ {}, 
        /* 1011 0000 */ {X86_OP_MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM}, 
        /* 1011 0001 */ {X86_OP_MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM}, 
        /* 1011 0010 */ {X86_OP_MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM}, 
        /* 1011 0011 */ {X86_OP_MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM}, 
        /* 1011 0100 */ {X86_OP_MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM}, 
        /* 1011 0101 */ {X86_OP_MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM}, 
        /* 1011 0110 */ {X86_OP_MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM}, 
        /* 1011 0111 */ {X86_OP_MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM}, 
        /* 1011 1000 */ {X86_OP_MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM | SIZE_WORD}, 
        /* 1011 1001 */ {X86_OP_MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM | SIZE_WORD}, 
        /* 1011 1010 */ {X86_OP_MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM | SIZE_WORD}, 
        /* 1011 1011 */ {X86_OP_MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM | SIZE_WORD}, 
        /* 1011 1100 */ {X86_OP_MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM | SIZE_WORD}, 
        /* 1011 1101 */ {X86_OP_MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM | SIZE_WORD}, 
        /* 1011 1110 */ {X86_OP_MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM | SIZE_WORD}, 
        /* 1011 1111 */ {X86_OP_MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM | SIZE_WORD}, 
        /* 1100 0000 */ {}, 
        /* 1100 0001 */ {}, 
        /* 1100 0010 */ {}, 
        /* 1100 0011 */ {}, 
        /* 1100 0100 */ {}, 
        /* 1100 0101 */ {}, 
        /* 1100 0110 */ {X86_OP_MOV_IMM_TO_RM, MOD_RM | IMM}, 
        /* 1100 0111 */ {X86_OP_MOV_IMM_TO_RM, MOD_RM | IMM | SIZE_WORD}, 
        /* 1100 1000 */ {}, 
        /* 1100 1001 */ {}, 
        /* 1100 1010 */ {}, 
        /* 1100 1011 */ {}, 
        /* 1100 1100 */ {}, 
        /* 1100 1101 */ {}, 
        /* 1100 1110 */ {}, 
        /* 1100 1111 */ {}, 
        /* 1101 0000 */ {}, 
        /* 1101 0001 */ {}, 
        /* 1101 0010 */ {}, 
        /* 1101 0011 */ {}, 
        /* 1101 0100 */ {}, 
        /* 1101 0101 */ {}, 
        /* 1101 0110 */ {}, 
        /* 1101 0111 */ {}, 
        /* 1101 1000 */ {}, 
        /* 1101 1001 */ {}, 
        /* 1101 1010 */ {}, 
        /* 1101 1011 */ {}, 
        /* 1101 1100 */ {}, 
        /* 1101 1101 */ {}, 
        /* 1101 1110 */ {}, 
        /* 1101 1111 */ {}, 
        /* 1110 0000 */ {X86_OP_LOOPNZ_LOOPNE, INC_IP_8BIT}, 
        /* 1110 0001 */ {X86_OP_LOOPZ_LOOPE, INC_IP_8BIT}, 
        /* 1110 0010 */ {X86_OP_LOOP, INC_IP_8BIT}, 
        /* 1110 0011 */ {X86_OP_JCXZ, INC_IP_8BIT}, 
        /* 1110 0100 */ {}, 
        /* 1110 0101 */ {}, 
        /* 1110 0110 */ {}, 
        /* 1110 0111 */ {}, 
        /* 1110 1000 */ {}, 
        /* 1110 1001 */ {}, 
        /* 1110 1010 */ {}, 
        /* 1110 1011 */ {}, 
        /* 1110 1100 */ {}, 
        /* 1110 1101 */ {}, 
        /* 1110 1110 */ {}, 
        /* 1110 1111 */ {}, 
        /* 1111 0000 */ {}, 
        /* 1111 0001 */ {}, 
        /* 1111 0010 */ {}, 
        /* 1111 0011 */ {}, 
        /* 1111 0100 */ {}, 
        /* 1111 0101 */ {}, 
        /* 1111 0110 */ {}, 
        /* 1111 0111 */ {}, 
        /* 1111 1000 */ {}, 
        /* 1111 1001 */ {}, 
        /* 1111 1010 */ {}, 
        /* 1111 1011 */ {}, 
        /* 1111 1100 */ {}, 
        /* 1111 1101 */ {}, 
        /* 1111 1110 */ {}, 
        /* 1111 1111 */ {}, 
    };

    u8 byte1 = *bytes++;

    OpMetadata firstByteMetadata = opMetadata[byte1];
    Assert(firstByteMetadata.op != 0);

    result.op = firstByteMetadata.op;
    DIRECTION regIsDest = DIRECTION((firstByteMetadata.flags & REG_IS_DST) > 0);
    WIDTH regWidth = WIDTH((firstByteMetadata.flags & SIZE_WORD) > 0);

    if(firstByteMetadata.flags & ACC) {
        X86_REG::Value reg = regWidth == WIDTH_WORD ? X86_REG::AX : X86_REG::AL;
        if(regIsDest) { // TODO: factor out
            result.operand1.reg = reg;
            result.operand1.flags = OPERAND_FLAGS_REG;
        } else {
            result.operand2.reg = reg;
            result.operand2.flags = OPERAND_FLAGS_REG;
        }
    }

    if(firstByteMetadata.flags & REG_BYTE1) {
        X86_REG::Value reg = decodeReg(byte1 & 0b111, regWidth);
        if(regIsDest) { // TODO: factor out
            result.operand1.reg = reg;
            result.operand1.flags = OPERAND_FLAGS_REG;
        } else {
            result.operand2.reg = reg;
            result.operand2.flags = OPERAND_FLAGS_REG;
        }
    }
    
    if(firstByteMetadata.flags & MOD_RM) {
        u8 byte2 = *bytes++; // mod reg rm

        if(firstByteMetadata.flags & REG_BYTE2) {
            X86_REG::Value reg = decodeReg((byte2 & 0b00111000) >> 3, regWidth);
            if(regIsDest) { // TODO: factor out
                result.operand1.reg = reg;
                result.operand1.flags = OPERAND_FLAGS_REG;
            } else {
                result.operand2.reg = reg;
                result.operand2.flags = OPERAND_FLAGS_REG;
            }
        } 
        
        if(firstByteMetadata.flags & PARTIAL_OP) {
            Assert((firstByteMetadata.flags & REG_BYTE2) == 0);
            Assert(result.op == X86_OP_PARTIAL_OP);
            Assert(firstByteMetadata.extOps);
            result.op = firstByteMetadata.extOps[(byte2 & 0b00111000) >> 3];
        }

        Mode mode = decodeMode(byte2 >> 6, byte2 & 0b111, regWidth);

        X86Operand* rmOperand;
        if(regIsDest) {
            rmOperand = &result.operand2;
        } else {
            rmOperand = &result.operand1;
        }

        if(mode.flags & MODE_FLAGS_REG) {
            rmOperand->reg = mode.memReg1;
            rmOperand->flags = turnOnFlags(rmOperand->flags, OPERAND_FLAGS_REG);
        } else {
            if(mode.flags & MODE_FLAGS_REG1_EFF_ADDR) { // mem = [reg + ?]
                rmOperand->memReg1 = mode.memReg1;
                rmOperand->flags = turnOnFlags(rmOperand->flags, OPERAND_FLAGS_MEM_REG1);
            }

            if(mode.flags & MODE_FLAGS_REG2_EFF_ADDR) { // mem = [memReg1 + memReg2 + ?]
                rmOperand->memReg2 = mode.memReg2;
                rmOperand->flags = turnOnFlags(rmOperand->flags, OPERAND_FLAGS_MEM_REG2);
            }

            if(mode.flags & (MODE_FLAGS_HAS_DISPLACE_8BIT | MODE_FLAGS_HAS_DISPLACE_16BIT)) {
                if(mode.flags & MODE_FLAGS_HAS_DISPLACE_8BIT) {
                    rmOperand->displacement = *(s8*)bytes++;
                } else {
                    rmOperand->displacement = *(s16*)bytes; bytes += 2;
                }
                rmOperand->flags = turnOnFlags(rmOperand->flags, OPERAND_FLAGS_DISPLACEMENT);
            }
        }
    }

    if(firstByteMetadata.flags & IMM) {
        if(firstByteMetadata.flags & SIZE_WORD) {
            result.operand2.flags = OPERAND_FLAGS_IMM_16BITS;
            if(firstByteMetadata.flags & SIGN_EXT) {
                result.operand2.immediate = *(s8*)bytes++;
            } else {
                result.operand2.immediate = *(s16*)bytes; bytes += 2;
            }
        } else {
            result.operand2.flags = OPERAND_FLAGS_IMM_8BITS;
            result.operand2.immediate = *(s8*)bytes++;
        }
    }

    if(firstByteMetadata.flags & MEM) {
        X86Operand* memOperand = regIsDest ? &result.operand2 : &result.operand1;
        memOperand->flags = OPERAND_FLAGS_DISPLACEMENT;
        memOperand->displacement = *(s16*)bytes; bytes += 2;
    }

    if(firstByteMetadata.flags & INC_IP_8BIT) {
        result.operand1.displacement = *(s8*)bytes++;
        result.operand1.flags = OPERAND_FLAGS_DISPLACEMENT;
        result.operand2.flags = OPERAND_FLAGS_NO_OPERAND;
    }

    result.nextByte = bytes;
    return result;
}

// TODO: Heavily consider just indexing everything in an array
const char* opName(X86_OP op) {
    if(op > x86_OP_MOV__START && op < X86_OP_ADD__END ) {
        return "mov";
    } else if(op > X86_OP_ADD__START && op < X86_OP_ADD__END) {
        return "add";
    } else if(op > X86_OP_SUB__START && op < X86_OP_SUB__END) {
        return "sub";
    } else if(op > X86_OP_CMP__START && op < X86_OP_CMP__END) {
        return "cmp";
    } else if(op > X86_OP_JMP__START && op < X86_OP_JMP__END) {
        switch(op) {
            case X86_OP_JE_JZ: return "je";
            case X86_OP_JL_JNGE: return "jl";
            case X86_OP_JLE_JNG: return "jle";
            case X86_OP_JB_JNAE: return "jb";
            case X86_OP_JBE_JNA: return "jbe";
            case X86_OP_JP_JPE: return "jp";
            case X86_OP_JO: return "jo";
            case X86_OP_JS: return "js";
            case X86_OP_JNE_JNZ: return "jnz";
            case X86_OP_JNL_JGE: return "jnl";
            case X86_OP_JNLE_JG: return "jg";
            case X86_OP_JNB_JAE: return "jnb";
            case X86_OP_JNBE_JA: return "ja";
            case X86_OP_JNP_JPO: return "jnp";
            case X86_OP_JNO: return "jno";
            case X86_OP_JNS: return "jns";
            case X86_OP_LOOP: return "loop";
            case X86_OP_LOOPZ_LOOPE: return "loopz";
            case X86_OP_LOOPNZ_LOOPNE: return "loopnz";
            case X86_OP_JCXZ: return "jcxz";
        }
    }

    InvalidCodePath return "";
}

void read8086Mnemonic(const char* asmFilePath) {
    u8* inputFileBuffer;
    long fileLen;

    FILE* inputFile = fopen(asmFilePath, "rb");
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
        printf("%s ", opName(op.op));

        auto writeAndPrintOperand = [](X86Operand operand, u8 prevOperandFlags) {
            if(operand.flags & OPERAND_FLAGS_REG) { // reg
                printf("%s", X86_REG::regName(operand.reg));
            } else if(checkAnyFlags(operand.flags, OPERAND_FLAGS_MEM_REG1 | OPERAND_FLAGS_DISPLACEMENT)) { // mem
                printf("[");
                if(operand.flags & OPERAND_FLAGS_MEM_REG1) { // effective address
                    printf("%s", X86_REG::regName(operand.memReg1));
                    if(operand.flags & OPERAND_FLAGS_MEM_REG2) {
                        printf(" + %s", X86_REG::regName(operand.memReg2));
                    }
                    if(operand.flags & OPERAND_FLAGS_DISPLACEMENT) {
                        operand.displacement >= 0 ? printf(" + %d", operand.displacement) : printf(" - %d", operand.displacement * -1);
                    }
                } else if(operand.flags & OPERAND_FLAGS_DISPLACEMENT) { // direct address
                    printf("%d", operand.displacement);
                }
                printf("]");
            } else if(checkAnyFlags(operand.flags, OPERAND_FLAGS_IMM_8BITS | OPERAND_FLAGS_IMM_16BITS)) { // imm
                bool operand1WasMem = checkAnyFlags(prevOperandFlags, OPERAND_FLAGS_MEM_REG1 | OPERAND_FLAGS_DISPLACEMENT);
                if(operand.flags & OPERAND_FLAGS_IMM_8BITS) {
                    printf(operand1WasMem ? "byte %d" : "%d", operand.immediate);
                } else if(operand.flags & OPERAND_FLAGS_IMM_16BITS) {
                    printf(operand1WasMem ? "word %d" : "%d", operand.immediate);
                }
            }
        };

        writeAndPrintOperand(op.operand1, 0);
        if((op.operand2.flags & OPERAND_FLAGS_NO_OPERAND) == 0) {
            printf(", ");
            writeAndPrintOperand(op.operand2, op.operand1.flags);
        }
    }
    free(inputFileBuffer);
}