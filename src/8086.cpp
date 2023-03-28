#define OUTPUT_FILE_HEADER "; Instruction decoding on the 8086 Homework by Connor Haskins\n\nbits 16\n\n"

// TODO: Consider just indexing everything in an array
const char* X86::opName(X86::OP op) {
    if(op > X86::MOV__START && op < X86::MOV__END ) {
        return "mov";
    } else if(op > X86::ADD__START && op < X86::ADD__END) {
        return "add";
    } else if(op > X86::SUB__START && op < X86::SUB__END) {
        return "sub";
    } else if(op > X86::CMP__START && op < X86::CMP__END) {
        return "cmp";
    } else if(op > X86::JMP__START && op < X86::JMP__END) {
        switch(op) {
            case X86::JE_JZ: return "je";
            case X86::JL_JNGE: return "jl";
            case X86::JLE_JNG: return "jle";
            case X86::JB_JNAE: return "jb";
            case X86::JBE_JNA: return "jbe";
            case X86::JP_JPE: return "jp";
            case X86::JO: return "jo";
            case X86::JS: return "js";
            case X86::JNE_JNZ: return "jnz";
            case X86::JNL_JGE: return "jnl";
            case X86::JNLE_JG: return "jg";
            case X86::JNB_JAE: return "jnb";
            case X86::JNBE_JA: return "ja";
            case X86::JNP_JPO: return "jnp";
            case X86::JNO: return "jno";
            case X86::JNS: return "jns";
            case X86::LOOP: return "loop";
            case X86::LOOPZ_LOOPE: return "loopz";
            case X86::LOOPNZ_LOOPNE: return "loopnz";
            case X86::JCXZ: return "jcxz";
        }
    } else {
        switch (op)
        {
            case X86::ADC_IMM_TO_RM: return "adc";
            case X86::SBB_IMM_FROM_RM: return "sbb";
            case X86::AND_IMM_WITH_RM: return "and";
            case X86::OR_IMM_WITH_RM: return "or";
            case X86::XOR_IMM_WITH_RM: return "xor";
        }
    }
    InvalidCodePath return "Unknown or unsupported op";
}

const char* X86::regName(REG val) {
    return regNames[val];
}

X86::REG decodeReg(u8 regCode, X86::WIDTH width) {
    assert(regCode >= 0b000 && regCode <= 0b111);
    return X86::REG(regCode + (width == X86::BYTE ? X86::AL : X86::AX));
}

X86::RegMem decodeRegMem(u8 mod, u8 rm, X86::WIDTH width) {
    assert(mod >= 0b00 && mod <= 0b11);
    assert(rm >= 0b000 && rm <= 0b111);

    X86::RegMem result{};

    // register to register
    if(mod == 0b11) {
        result.reg = decodeReg(rm, width);
        result.flags = setFlags(result.flags, X86::RM_FLAGS_REG);
        return result;
    }

    // direct access special case
    if(rm == 0b110 && mod == 0b00) {
        result.flags = X86::RM_FLAGS_HAS_DISPLACE_16BIT;
        return result;
    }

    // load effective memory address to register
    X86::REG effAddrRegs0to4[4][2] = {
        {X86::BX, X86::SI},
        {X86::BX, X86::DI},
        {X86::BP, X86::SI},
        {X86::BP, X86::DI}
    };
    X86::REG effAddrRegs5to7[4] { X86::SI, X86::DI, X86::BP, X86::BX };

    if(rm < 0b100) {
        result.memReg1 = effAddrRegs0to4[rm][0];
        result.memReg2 = effAddrRegs0to4[rm][1];
        result.flags = setFlags(result.flags, X86::RM_FLAGS_REG1_EFF_ADDR | X86::RM_FLAGS_REG2_EFF_ADDR);
    } else {
        result.memReg1 = effAddrRegs5to7[rm - 4];
        result.flags = setFlags(result.flags, X86::RM_FLAGS_REG1_EFF_ADDR);
    }
    
    if(mod == 0b01) {
        result.flags = setFlags(result.flags, X86::RM_FLAGS_HAS_DISPLACE_8BIT);
    } else if(mod == 0b10) {
        result.flags = setFlags(result.flags, X86::RM_FLAGS_HAS_DISPLACE_16BIT);
    }

    return result;
}

X86::DecodedOp decodeOp(u8* bytes) {

    X86::DecodedOp result{};
    u8 byte1 = *bytes++;

    X86::OpMetadata firstByteMetadata = X86::opMetadata[byte1];
    assert(firstByteMetadata.op != 0);

    result.op = firstByteMetadata.op;
    bool regIsDst = firstByteMetadata.flags & X86::REG_IS_DST;
    X86::WIDTH regWidth = X86::WIDTH((firstByteMetadata.flags & X86::WIDTH_WORD) > 0);
    
    // Note: there may not be a reg operand but if there is, it will be decided by regIsDst
    X86::Operand* regOperand = regIsDst ? &result.operand1 : &result.operand2;

    if(firstByteMetadata.flags & X86::ACC) {
        regOperand->reg = regWidth == X86::WORD ? X86::AX : X86::AL;
        regOperand->flags = X86::OPERAND_REG;
    }

    if(firstByteMetadata.flags & X86::REG_BYTE1) {
        regOperand->reg = decodeReg(byte1 & 0b111, regWidth);
        regOperand->flags = X86::OPERAND_REG;
    }
    
    if(firstByteMetadata.flags & X86::MOD_RM) {
        u8 byte2 = *bytes++; // mod reg rm

        X86::Operand* rmOperand = regIsDst ? &result.operand2 : &result.operand1;

        if(firstByteMetadata.flags & X86::REG_BYTE2) {
            regOperand->reg = decodeReg((byte2 & 0b00111000) >> 3, regWidth);
            regOperand->flags = X86::OPERAND_REG;
        } 
        
        if(firstByteMetadata.flags & X86::ADDTL_OP_CODE) {
            assert((firstByteMetadata.flags & X86::REG_BYTE2) == 0);
            assert(result.op == X86::PARTIAL_OP);
            assert(firstByteMetadata.extOps);
            result.op = firstByteMetadata.extOps[(byte2 & 0b00111000) >> 3];
        }

        X86::RegMem regMem = decodeRegMem(byte2 >> 6, byte2 & 0b111, regWidth);

        if(regMem.isReg()) {
            rmOperand->reg = regMem.reg;
            rmOperand->flags = setFlags(rmOperand->flags, X86::OPERAND_REG);
        } else {
            if(regMem.flags & X86::RM_FLAGS_REG1_EFF_ADDR) { // mem = [reg + ?]
                rmOperand->memReg1 = regMem.memReg1;
                rmOperand->flags = setFlags(rmOperand->flags, X86::OPERAND_MEM_REG1);
            }

            if(regMem.flags & X86::RM_FLAGS_REG2_EFF_ADDR) { // mem = [memReg1 + memReg2 + ?]
                rmOperand->memReg2 = regMem.memReg2;
                rmOperand->flags = setFlags(rmOperand->flags, X86::OPERAND_MEM_REG2);
            }

            if(regMem.flags & X86::RM_FLAGS_HAS_DISPLACE_8BIT) {
                rmOperand->displacement = *(s8*)bytes++;
                rmOperand->flags = setFlags(rmOperand->flags, X86::OPERAND_DISPLACEMENT);
            } else if(regMem.flags & X86::RM_FLAGS_HAS_DISPLACE_16BIT) {
                rmOperand->displacement = *(s16*)bytes; bytes += 2;
                rmOperand->flags = setFlags(rmOperand->flags, X86::OPERAND_DISPLACEMENT);
            }
        }
    }

    if(firstByteMetadata.flags & X86::IMM) {
        if(firstByteMetadata.flags & X86::WIDTH_WORD) {
            result.operand2.flags = X86::OPERAND_IMM_16BITS;
            if(firstByteMetadata.flags & X86::SIGN_EXT) {
                result.operand2.immediate = *(s8*)bytes++;
            } else {
                result.operand2.immediate = *(s16*)bytes; bytes += 2;
            }
        } else {
            result.operand2.flags = X86::OPERAND_IMM_8BITS;
            result.operand2.immediate = *(s8*)bytes++;
        }
    }

    if(firstByteMetadata.flags & X86::MEM) {
        X86::Operand* memOperand = regIsDst ? &result.operand2 : &result.operand1;
        memOperand->flags = X86::OPERAND_DISPLACEMENT;
        memOperand->displacement = *(s16*)bytes; bytes += 2;
    }

    if(firstByteMetadata.flags & X86::INC_IP_8BIT) {
        result.operand1.displacement = *(s8*)bytes++;
        result.operand1.flags = X86::OPERAND_DISPLACEMENT;
        result.operand2.flags = X86::OPERAND_NO_OPERAND;
    }

    result.nextByte = bytes;
    return result;
}

void printOp(X86::DecodedOp op) {
    printf("%s ", X86::opName(op.op));

    auto writeAndPrintOperand = [](X86::Operand operand, X86::Operand* prevOperand) {
        if(operand.isReg()) {
            printf("%s", X86::regName(operand.reg));
        } else if(operand.isMem()) {
            printf("[");
            if(operand.flags & X86::OPERAND_MEM_REG1) { // effective address
                printf("%s", X86::regName(operand.memReg1));
                if(operand.flags & X86::OPERAND_MEM_REG2) {
                    printf(" + %s", X86::regName(operand.memReg2));
                }
                if(operand.flags & X86::OPERAND_DISPLACEMENT) {
                    operand.displacement >= 0 ? printf(" + %d", operand.displacement) : printf(" - %d", operand.displacement * -1);
                }
            } else if(operand.flags & X86::OPERAND_DISPLACEMENT) { // direct address
                printf("%d", operand.displacement);
            }
            printf("]");
        } else if(operand.isImm()) {
            bool operand1WasMem = prevOperand ? prevOperand->isMem() : false;
            if(operand.flags & X86::OPERAND_IMM_8BITS) {
                printf(operand1WasMem ? "byte %d" : "%d", operand.immediate);
            } else if(operand.flags & X86::OPERAND_IMM_16BITS) {
                printf(operand1WasMem ? "word %d" : "%d", operand.immediate);
            }
        }
    };

    if(op.operand2.flags & X86::OPERAND_NO_OPERAND) { // assume some kind of jmp
        printf("$%+d", op.operand1.displacement + 2);
    } else { 
        writeAndPrintOperand(op.operand1, nullptr);
        printf(", ");
        writeAndPrintOperand(op.operand2, &op.operand1);
    }
}

X86::CpuState executeOp(X86::DecodedOp op, X86::CpuState state) {
    printf(" ; ");

    switch(op.op) {
        case X86::MOV_IMM_TO_REG: {
            u16 prevVal = state.regVal(op.operand1.reg);
            state.regSet(op.operand1.reg, op.operand2.immediate);
            u16 newVal = state.regVal(op.operand1.reg);
            printf("%s:0x%04x->0x%04x", regName(op.operand1.reg), prevVal, newVal);
            break;
        }
        case X86::MOV_RM_TO_FROM_REG: {
            if(op.operand1.flags & X86::OPERAND_REG) { // reg is dst
                u16 prevVal = state.regVal(op.operand1.reg);
                if(op.operand2.flags & X86::OPERAND_REG) { // reg to reg
                    state.regSet(op.operand1.reg, state.regVal(op.operand2.reg));
                    u16 newVal = state.regVal(op.operand1.reg);
                    printf("%s:0x%04x->0x%04x", regName(op.operand1.reg), prevVal, newVal);
                    break;
                }
            }
            // TODO: Implement more RM to/from REG. Put back break. Currently fall through to error.
        }
        default:
            printf("ERROR: Executing op %s not yet implemented!", X86::opName(op.op));
    }

    return state;
}

void printFinalRegisters(X86::CpuState state) {
    printf(
        "\n;Final registers:\n"
        ";\tax: 0x%04x (%d)\n"
        ";\tbx: 0x%04x (%d)\n"
        ";\tcx: 0x%04x (%d)\n"
        ";\tdx: 0x%04x (%d)\n"
        ";\tsp: 0x%04x (%d)\n"
        ";\tbp: 0x%04x (%d)\n"
        ";\tsi: 0x%04x (%d)\n"
        ";\tdi: 0x%04x (%d)\n",
        state.regs.ax, state.regs.ax,
        state.regs.bx, state.regs.bx,
        state.regs.cx, state.regs.cx,
        state.regs.dx, state.regs.dx,
        state.regs.sp, state.regs.sp,
        state.regs.bp, state.regs.bp,
        state.regs.si, state.regs.si,
        state.regs.di, state.regs.di);
}

void decode8086Binary(const char* asmFilePath, bool execute) {
    u8* inputFileBuffer;
    long fileLen;

    FILE* inputFile = fopen(asmFilePath, "rb");
    if(!inputFile) {
        printf("ERROR: Could not find file [%s]\nABORTING", asmFilePath);
        return;
    }
    fseek(inputFile, 0, SEEK_END);
    fileLen = ftell(inputFile);
    rewind(inputFile);

    inputFileBuffer = (u8 *)malloc(fileLen * sizeof(u8));
    fread(inputFileBuffer, fileLen, 1, inputFile);
    fclose(inputFile);
    inputFile = nullptr;

    printf(OUTPUT_FILE_HEADER);
    
    X86::CpuState cpuState{};
    u8* lastByte = inputFileBuffer + fileLen;
    u8* currentByte = inputFileBuffer;
    while(currentByte < lastByte) {
        X86::DecodedOp op = decodeOp(currentByte); 
        printOp(op);
        if(execute) { cpuState = executeOp(op, cpuState); }
        printf("\n");
        currentByte = op.nextByte;
    }
    if(execute) { printFinalRegisters(cpuState); }
    free(inputFileBuffer);
}