namespace X86 {
    enum OP {
        UNDEFINED = 0,
        PARTIAL_OP,

        MOV__START,
        MOV_RM_TO_FROM_REG,
        MOV_IMM_TO_RM,
        MOV_IMM_TO_REG,
        MOV_MEM_TO_ACC,
        MOV_ACC_TO_MEM,
        MOV__END,

        ADD__START,
        ADD_RM_TO_FROM_REG,
        ADD_IMM_TO_RM,
        ADD_IMM_TO_ACC,
        ADD__END,

        SUB__START,
        SUB_RM_TO_FROM_REG,
        SUB_IMM_FROM_RM,
        SUB_IMM_FROM_ACC,
        SUB__END,

        CMP__START,
        CMP_RM_AND_REG,
        CMP_IMM_AND_RM,
        CMP_IMM_AND_ACC,
        CMP__END,

        JMP__START,
        JE_JZ,
        JL_JNGE,
        JLE_JNG,
        JB_JNAE,
        JBE_JNA,
        JP_JPE,
        JO,
        JS,
        JNE_JNZ,
        JNL_JGE,
        JNLE_JG,
        JNB_JAE,
        JNBE_JA,
        JNP_JPO,
        JNO,
        JNS,
        LOOP,
        LOOPZ_LOOPE,
        LOOPNZ_LOOPNE,
        JCXZ,
        JMP__END,

        ADC_IMM_TO_RM,
        SBB_IMM_FROM_RM,
        AND_IMM_WITH_RM,
        OR_IMM_WITH_RM,
        XOR_IMM_WITH_RM,
    };

    enum REG { // NOTE: ORDER MATTERS
        AX = 0, CX, DX, BX,
        SP, BP, SI, DI,

        AL, CL, DL, BL,
        AH, CH, DH, BH
    };
    bool regIsWide(REG reg) { return reg < AL; };

    struct CpuState {
        enum FLAGS {
            CF = 1 << 0, // Carry Flag
            PF = 1 << 2, // Parity Flag
            AF = 1 << 4, // Auxiliary Carry Flag
            ZF = 1 << 6, // Zero Flag
            SF = 1 << 7, // Sign Flag
            TF = 1 << 8, // Trap Flag
            IF = 1 << 9, // Interrupt Enable Flag
            DF = 1 << 10, // Direction Flag
            OF = 1 << 11, // Overflow Flag
        };

        static const u8 regEnumToOffset[16];
        struct { // NOTE: ORDER MATTERS
            u16 ax; // [u8 al, u8 ah]
            u16 cx; // [u8 cl, u8 ch]
            u16 dx; // [u8 dl, u8 dh]
            u16 bx; // [u8 bl, u8 bh]
            u16 sp;
            u16 bp;
            u16 si;
            u16 di;
            
            u16 cs;
            u16 ds;
            u16 ss;
            u16 es;
        } regs;
        u16 flags;

        u16 regVal(REG reg) {
            u8 offset = regEnumToOffset[reg];
            u8* regPtr = (u8*)&regs + offset;
            return regIsWide(reg) ? *(u16*)regPtr : *regPtr;
        }

        void regSet(REG reg, u16 val) {
            u8 offset = regEnumToOffset[reg];
            u8* regPtr = (u8*)&regs + offset;
            if(regIsWide(reg)) {
                u16* reg16Ptr = (u16*)regPtr;
                *reg16Ptr = val;
            } else {
                *regPtr = (u8)val;
            }
        }

        void clearFlags() { flags = 0; }

        void setFlag(FLAGS flag) { flags |= flag; }

        static void printFlags(u16 flags) {
            if(flags & CF) { printf("C"); } 
            if(flags & PF) { printf("P"); } 
            if(flags & AF) { printf("A"); } 
            if(flags & ZF) { printf("Z"); } 
            if(flags & SF) { printf("S"); } 
            if(flags & TF) { printf("T"); } 
            if(flags & IF) { printf("I"); } 
            if(flags & DF) { printf("D"); } 
            if(flags & OF) { printf("O"); }
        };
    };
    const u8 CpuState::regEnumToOffset[16] = {
        0, 2, 4, 6, // ax, cx, dx, bx 
        8, 10, 12, 14, // sp, bp, si, di

        0, 2, 4, 6, // al, cl, dl, bl
        1, 3, 5, 7, // ah, ch, dh, bh
    };

    enum WIDTH {
        BYTE = 0,
        WORD = 1
    };

    enum RM_FLAGS {
        RM_FLAGS_REG = 1 << 0,
        RM_FLAGS_REG1_EFF_ADDR = 1 << 1,
        RM_FLAGS_REG2_EFF_ADDR = 1 << 2,
        RM_FLAGS_HAS_DISPLACE_8BIT = 1 << 3,
        RM_FLAGS_HAS_DISPLACE_16BIT = 1 << 4
    };

    enum OP_METADATA : u32 {
        REG_IS_DST      = 1 << 0, // if 0, reg is src where applicable
        SIGN_EXT        = 1 << 1, // if 0, values are not sign extended
        WIDTH_WORD      = 1 << 2, // if 0, values are 1 byte where applicable
        REG_BYTE1       = 1 << 3, // There's a reg code in byte 1
        MOD_RM          = 1 << 4, // Three's a MOD R/M in byte 2
        REG_BYTE2       = 1 << 5, // There's a reg code in byte 2
        ADDTL_OP_CODE   = 1 << 6, // There's an addt'l op code in byte 2
        IMM             = 1 << 7, // Immediate data follows
        MEM             = 1 << 8, // Memory data follows
        ACC             = 1 << 9, // Op uses the accumulator (AX/AL)
        INC_IP_8BIT     = 1 << 10,// Signed increment to instruction pointer (jumps)
    };

    enum OPERAND_FLAGS {
        OPERAND_REG = 1 << 0,
        OPERAND_MEM_REG1 = 1 << 1,
        OPERAND_MEM_REG2 = 1 << 2,
        OPERAND_DISPLACEMENT = 1 << 3,
        OPERAND_IMM_8BITS = 1 << 4,
        OPERAND_IMM_16BITS = 1 << 5,
        OPERAND_NO_OPERAND = 1 << 6,
    };

    struct RegMem {
        union {
            REG reg;
            REG memReg1;
        };
        REG memReg2;
        u8 flags;
        bool isReg(){ return flags & RM_FLAGS_REG; }
    };

    struct Operand {
        union {
            REG reg;
            REG memReg1;
        };
        REG memReg2;

        union {
            s16 displacement;
            s16 immediate;
        };
        u8 flags;
        bool isMem() { return checkAnyFlags(flags, OPERAND_MEM_REG1 | OPERAND_DISPLACEMENT); }
        bool isImm() { return checkAnyFlags(flags, OPERAND_IMM_8BITS | OPERAND_IMM_16BITS); }
        bool isReg() { return flags & OPERAND_REG; }
    };

    struct OpMetadata { // NOTE: member order matters and is depended on in initializations
        OP op;
        u32 flags;
        OP* extOps;
    };

    struct DecodedOp {
        OP op;
        Operand operand1;
        Operand operand2;
        u8* nextByte;
    };

    const char* regNames[] = { // NOTE: ORDER FOLLOWS ENUMS
            "ax","cx", "dx", "bx",
            "sp", "bp", "si", "di",

            "al","cl", "dl", "bl",
            "ah","ch", "dh", "bh"
    };

    const char* regName(REG val);
    const char* opName(X86::OP op);

    OP extOps_1000_00xx[8] {
        ADD_IMM_TO_RM, // 000
        OR_IMM_WITH_RM, // 001
        ADC_IMM_TO_RM, // 010
        SBB_IMM_FROM_RM, // 011
        AND_IMM_WITH_RM, // 100
        SUB_IMM_FROM_RM, // 101
        XOR_IMM_WITH_RM, // 110
        CMP_IMM_AND_RM, // 111
    };
    
    OpMetadata opMetadata[256] = {
    /* 0000 0000 */ {X86::ADD_RM_TO_FROM_REG, MOD_RM | REG_BYTE2}, 
    /* 0000 0001 */ {X86::ADD_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | WIDTH_WORD}, 
    /* 0000 0010 */ {X86::ADD_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | REG_IS_DST}, 
    /* 0000 0011 */ {X86::ADD_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | REG_IS_DST | WIDTH_WORD}, 
    /* 0000 0100 */ {X86::ADD_IMM_TO_ACC, ACC | IMM | REG_IS_DST}, 
    /* 0000 0101 */ {X86::ADD_IMM_TO_ACC, ACC | IMM | REG_IS_DST | WIDTH_WORD}, 
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
    /* 0010 1000 */ {X86::SUB_RM_TO_FROM_REG, MOD_RM | REG_BYTE2}, 
    /* 0010 1001 */ {X86::SUB_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | WIDTH_WORD}, 
    /* 0010 1010 */ {X86::SUB_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | REG_IS_DST}, 
    /* 0010 1011 */ {X86::SUB_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | REG_IS_DST | WIDTH_WORD}, 
    /* 0010 1100 */ {X86::SUB_IMM_FROM_ACC, ACC | IMM | REG_IS_DST}, 
    /* 0010 1101 */ {X86::SUB_IMM_FROM_ACC, ACC | IMM | REG_IS_DST | WIDTH_WORD}, 
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
    /* 0011 1000 */ {X86::CMP_RM_AND_REG, MOD_RM | REG_BYTE2}, 
    /* 0011 1001 */ {X86::CMP_RM_AND_REG, MOD_RM | REG_BYTE2 | WIDTH_WORD}, 
    /* 0011 1010 */ {X86::CMP_RM_AND_REG, MOD_RM | REG_BYTE2 | REG_IS_DST}, 
    /* 0011 1011 */ {X86::CMP_RM_AND_REG, MOD_RM | REG_BYTE2 | REG_IS_DST | WIDTH_WORD}, 
    /* 0011 1100 */ {X86::CMP_IMM_AND_ACC, ACC | IMM | REG_IS_DST}, 
    /* 0011 1101 */ {X86::CMP_IMM_AND_ACC, ACC | IMM | REG_IS_DST | WIDTH_WORD},
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
    /* 0111 0000 */ {X86::JO, INC_IP_8BIT}, 
    /* 0111 0001 */ {X86::JNO, INC_IP_8BIT}, 
    /* 0111 0010 */ {X86::JB_JNAE, INC_IP_8BIT}, 
    /* 0111 0011 */ {X86::JNB_JAE, INC_IP_8BIT}, 
    /* 0111 0100 */ {X86::JE_JZ, INC_IP_8BIT}, 
    /* 0111 0101 */ {X86::JNE_JNZ, INC_IP_8BIT}, 
    /* 0111 0110 */ {X86::JBE_JNA, INC_IP_8BIT}, 
    /* 0111 0111 */ {X86::JNBE_JA, INC_IP_8BIT}, 
    /* 0111 1000 */ {X86::JS, INC_IP_8BIT}, 
    /* 0111 1001 */ {X86::JNS, INC_IP_8BIT}, 
    /* 0111 1010 */ {X86::JP_JPE, INC_IP_8BIT}, 
    /* 0111 1011 */ {X86::JNP_JPO, INC_IP_8BIT}, 
    /* 0111 1100 */ {X86::JL_JNGE, INC_IP_8BIT}, 
    /* 0111 1101 */ {X86::JNL_JGE, INC_IP_8BIT}, 
    /* 0111 1110 */ {X86::JLE_JNG, INC_IP_8BIT}, 
    /* 0111 1111 */ {X86::JNLE_JG, INC_IP_8BIT}, 
    /* 1000 0000 */ {X86::PARTIAL_OP, MOD_RM | ADDTL_OP_CODE | IMM, extOps_1000_00xx},
    /* 1000 0001 */ {X86::PARTIAL_OP, MOD_RM | ADDTL_OP_CODE | IMM | WIDTH_WORD, extOps_1000_00xx},
    /* 1000 0010 */ {X86::PARTIAL_OP, MOD_RM | ADDTL_OP_CODE | IMM | SIGN_EXT, extOps_1000_00xx},
    /* 1000 0011 */ {X86::PARTIAL_OP, MOD_RM | ADDTL_OP_CODE | IMM | SIGN_EXT | WIDTH_WORD, extOps_1000_00xx},
    /* 1000 0100 */ {}, 
    /* 1000 0101 */ {}, 
    /* 1000 0110 */ {}, 
    /* 1000 0111 */ {}, 
    /* 1000 1000 */ {X86::MOV_RM_TO_FROM_REG, MOD_RM | REG_BYTE2}, 
    /* 1000 1001 */ {X86::MOV_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | WIDTH_WORD}, 
    /* 1000 1010 */ {X86::MOV_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | REG_IS_DST}, 
    /* 1000 1011 */ {X86::MOV_RM_TO_FROM_REG, MOD_RM | REG_BYTE2 | REG_IS_DST | WIDTH_WORD}, 
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
    /* 1010 0000 */ {X86::MOV_MEM_TO_ACC, ACC | MEM | REG_IS_DST}, 
    /* 1010 0001 */ {X86::MOV_MEM_TO_ACC, ACC | MEM | REG_IS_DST | WIDTH_WORD}, 
    /* 1010 0010 */ {X86::MOV_ACC_TO_MEM, ACC | MEM}, 
    /* 1010 0011 */ {X86::MOV_ACC_TO_MEM, ACC | MEM | WIDTH_WORD},
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
    /* 1011 0000 */ {X86::MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM}, 
    /* 1011 0001 */ {X86::MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM}, 
    /* 1011 0010 */ {X86::MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM}, 
    /* 1011 0011 */ {X86::MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM}, 
    /* 1011 0100 */ {X86::MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM}, 
    /* 1011 0101 */ {X86::MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM}, 
    /* 1011 0110 */ {X86::MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM}, 
    /* 1011 0111 */ {X86::MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM}, 
    /* 1011 1000 */ {X86::MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM | WIDTH_WORD}, 
    /* 1011 1001 */ {X86::MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM | WIDTH_WORD}, 
    /* 1011 1010 */ {X86::MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM | WIDTH_WORD}, 
    /* 1011 1011 */ {X86::MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM | WIDTH_WORD}, 
    /* 1011 1100 */ {X86::MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM | WIDTH_WORD}, 
    /* 1011 1101 */ {X86::MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM | WIDTH_WORD}, 
    /* 1011 1110 */ {X86::MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM | WIDTH_WORD}, 
    /* 1011 1111 */ {X86::MOV_IMM_TO_REG, REG_BYTE1 | REG_IS_DST | IMM | WIDTH_WORD}, 
    /* 1100 0000 */ {}, 
    /* 1100 0001 */ {}, 
    /* 1100 0010 */ {}, 
    /* 1100 0011 */ {}, 
    /* 1100 0100 */ {}, 
    /* 1100 0101 */ {}, 
    /* 1100 0110 */ {X86::MOV_IMM_TO_RM, MOD_RM | IMM}, 
    /* 1100 0111 */ {X86::MOV_IMM_TO_RM, MOD_RM | IMM | WIDTH_WORD}, 
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
    /* 1110 0000 */ {X86::LOOPNZ_LOOPNE, INC_IP_8BIT}, 
    /* 1110 0001 */ {X86::LOOPZ_LOOPE, INC_IP_8BIT}, 
    /* 1110 0010 */ {X86::LOOP, INC_IP_8BIT}, 
    /* 1110 0011 */ {X86::JCXZ, INC_IP_8BIT}, 
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
}