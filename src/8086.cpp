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
    u8* byte1 = bytes++;

    X86::OpMetadata firstByteMetadata = X86::opMetadata[*byte1];
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
        regOperand->reg = decodeReg(*byte1 & 0b111, regWidth);
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

    result.sizeInBytes = (u8)(bytes - byte1);
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

X86::CpuState executeOp(X86::DecodedOp decOp, X86::CpuState state) {

    struct FUNCS {
        static void printRegChange(X86::CpuState *state, X86::REG reg, u16 oldVal) {
            const char* regChangeFmtStr = "%s:0x%04x->0x%04x";
            u16 newVal = state->regVal(reg);
            printf(regChangeFmtStr, regName(reg), oldVal, newVal);
            printf(" ip:0x%04x", state->regs.ip);
        }

        static void printFlagsChange(u16 oldFlags, u16 newFlags) {
            printf(" flags: ");
            X86::CpuState::printFlags(oldFlags);
            printf("->");
            X86::CpuState::printFlags(newFlags);
        }

        static void updateSZPFlagsRegister(X86::CpuState* state, u16 val) {
            if(val & (1 << 15)) { state->setFlag(X86::CpuState::SF); 
            } else if(val == 0) { state->setFlag(X86::CpuState::ZF); }
            if(parity(val)) { state->setFlag(X86::CpuState::PF); }
        }

        static void executeAdd(X86::CpuState* state, X86::REG reg, u16 val) {
            u16 oldFlags = state->flags;
            state->clearFlags();
            if(regIsWide(reg)) {
                u16 prevVal = state->regVal(reg);
                u16 newVal = prevVal + val;
                state->regSet(reg, newVal);
                printRegChange(state, reg, prevVal);
            } else {
                u8 prevVal = (u8)state->regVal(reg);
                u8 newVal = prevVal + (u8)val;
                state->regSet(reg, newVal);
                printRegChange(state, reg, prevVal);
            }
            updateSZPFlagsRegister(state, state->regVal(reg));
            printFlagsChange(oldFlags, state->flags);
        }

        static void executeSub(X86::CpuState* state, X86::REG reg, u16 val) {
            executeAdd(state, reg, (u16)0 - val);
        }

        static void executeCmp(X86::CpuState* state, u16 val1, u16 val2) {
            u16 oldFlags = state->flags;
            state->clearFlags();
            u16 cmpRes = val1 - val2;
            updateSZPFlagsRegister(state, cmpRes);
            printFlagsChange(oldFlags, state->flags);
        }

        static void executeMov(X86::CpuState* state, X86::REG reg, u16 val) {
            if(regIsWide(reg)) {
                u16 prevVal = state->regVal(reg);
                state->regSet(reg, val);
                printRegChange(state, reg, prevVal);
            } else {
                u8 prevVal = (u8)state->regVal(reg);
                state->regSet(reg, (u8)val);
                printRegChange(state, reg, prevVal);
            }
        }

        static void executeJmp(X86::CpuState* state, s16 displacement) {
            state->regs.ip += displacement;
        }
    };

    state.regs.ip += decOp.sizeInBytes;

    printf(" ; ");

    // Empty switch statement for all op.op cases
    switch(decOp.op) {
        case X86::MOV_IMM_TO_REG: {
            FUNCS::executeMov(&state, decOp.operand1.reg, decOp.operand2.immediate);
            break;
        }
        case X86::MOV_RM_TO_FROM_REG: {
            if(decOp.operand1.isReg()) {
                if(decOp.operand2.isReg()) {
                    FUNCS::executeMov(&state, decOp.operand1.reg, state.regVal(decOp.operand2.reg));
                } else if(decOp.operand2.isMem()) {
                    // TODO
                    printf("ERROR: Unimplemented mov to reg from mem");
                } else if(decOp.operand2.isImm()) {
                    FUNCS::executeMov(&state, decOp.operand1.reg, decOp.operand2.immediate);
                }
            } else { // mov to mem
                // TODO
                printf("ERROR: Unimplemented mov to mem");
            }
            break;
        }

        case X86::ADD_IMM_TO_ACC: {
            FUNCS::executeAdd(&state, decOp.operand1.reg, decOp.operand2.immediate);
            break;
        }

        case X86::ADD_IMM_TO_RM: {
            if(decOp.operand1.isReg()) {
                FUNCS::executeAdd(&state, decOp.operand1.reg, decOp.operand2.immediate);
            } else if(decOp.operand1.isMem()) {
                // TODO
                printf("ERROR: Unimplemented add to mem from imm");
            }
            break;
        }

        case X86::SUB_IMM_FROM_ACC: {
            FUNCS::executeSub(&state, decOp.operand1.reg, decOp.operand2.immediate);
            break;
        }

        case X86::SUB_IMM_FROM_RM:{
            if(decOp.operand1.isReg()) {
                FUNCS::executeSub(&state, decOp.operand1.reg, decOp.operand2.immediate);
            } else if(decOp.operand1.isMem()) {
                printf("ERROR: Unimplemented sub to mem from imm");
            }
            break;
        }

        case X86::SUB_RM_TO_FROM_REG: {
            if(decOp.operand1.isReg()) {
                if(decOp.operand2.isReg()) {
                    FUNCS::executeSub(&state, decOp.operand1.reg, state.regVal(decOp.operand2.reg));
                } else if(decOp.operand2.isMem()) {
                    // TODO
                    printf("ERROR: Unimplemented sub [reg - mem]");
                } else if(decOp.operand2.isImm()) {
                    FUNCS::executeSub(&state, decOp.operand1.reg, decOp.operand2.immediate);
                }
            } else if(decOp.operand1.isMem()) {
                // TODO
                printf("ERROR: Unimplemented sub to mem");
            }
            break;
        }

        case X86::CMP_IMM_AND_ACC: {
            FUNCS::executeCmp(&state, state.regVal(decOp.operand1.reg), decOp.operand2.immediate);
            break;
        }

        case X86::CMP_IMM_AND_RM: {
            u16 val1{}, val2{};
            if(decOp.operand1.isReg()) {
                val1 = state.regVal(decOp.operand1.reg);
            } else if(decOp.operand1.isMem()) {
                // TODO
                val1 = 1 << 14;
                printf("ERROR: Unimplemented cmp with mem");
            } else if(decOp.operand1.isImm()) {
                val1 = decOp.operand1.immediate;
            }

            if(decOp.operand2.isReg()) {
                val2 = state.regVal(decOp.operand2.reg);
            } else if(decOp.operand2.isMem()) {
                // TODO
                val2 = 1 << 14;
                printf("ERROR: Unimplemented cmp with mem");
            } else if(decOp.operand2.isImm()) {
                val2 = decOp.operand2.immediate;
            }

            FUNCS::executeCmp(&state, val1, val2);
            break;
        }

        case X86::CMP_RM_AND_REG: {
            u16 val1{}, val2{};
            if(decOp.operand1.isReg()) {
                val1 = state.regVal(decOp.operand1.reg);
            } else if(decOp.operand1.isMem()) {
                // TODO
                val1 = 1 << 14;
                printf("ERROR: Unimplemented cmp with mem");
            }

            if(decOp.operand2.isReg()) {
                val2 = state.regVal(decOp.operand2.reg);
            } else if(decOp.operand2.isMem()) {
                // TODO
                val2 = 1 << 14;
                printf("ERROR: Unimplemented cmp with mem");
            }

            FUNCS::executeCmp(&state, val1, val2);
            break;
        }

        case X86::JE_JZ: {
            if(state.flags & X86::CpuState::FLAGS::ZF) { FUNCS::executeJmp(&state, decOp.operand1.displacement);}
            break;
        }

        case X86::JL_JNGE: {
            if(state.flags & X86::CpuState::FLAGS::SF) { FUNCS::executeJmp(&state, decOp.operand1.displacement);}
            break;
        }

        case X86::JLE_JNG: {
            if(state.flags & (X86::CpuState::FLAGS::ZF | X86::CpuState::FLAGS::SF)) { FUNCS::executeJmp(&state, decOp.operand1.displacement);}
            break;
        }

        case X86::JB_JNAE: {
            if(state.flags & X86::CpuState::FLAGS::CF) { FUNCS::executeJmp(&state, decOp.operand1.displacement);}
            break;
        }

        case X86::JBE_JNA: {
            if(state.flags & X86::CpuState::FLAGS::CF || state.flags & X86::CpuState::FLAGS::ZF) { FUNCS::executeJmp(&state, decOp.operand1.displacement); }
            break;
        }

        case X86::JP_JPE: {
            if(state.flags & X86::CpuState::FLAGS::PF) { FUNCS::executeJmp(&state, decOp.operand1.displacement);}
            break;
        }

        case X86::JO: {
            if(state.flags & X86::CpuState::FLAGS::OF) { FUNCS::executeJmp(&state, decOp.operand1.displacement);}
            break;
        }

        case X86::JS: {
            if(state.flags & X86::CpuState::FLAGS::SF) { FUNCS::executeJmp(&state, decOp.operand1.displacement);}
            break;
        }

        case X86::JNE_JNZ: {
            if(!(state.flags & X86::CpuState::FLAGS::ZF)) { FUNCS::executeJmp(&state, decOp.operand1.displacement);}
            break;
        }

        case X86::JNL_JGE: {
            if(XOR(state.flags & X86::CpuState::FLAGS::SF, state.flags & X86::CpuState::FLAGS::OF)) { 
                FUNCS::executeJmp(&state, decOp.operand1.displacement);
            }
            break;
        }

        case X86::JNLE_JG: {
            if(!(XOR(state.flags & X86::CpuState::FLAGS::SF, state.flags & X86::CpuState::FLAGS::OF) || (state.flags & X86::CpuState::FLAGS::ZF))) { 
                FUNCS::executeJmp(&state, decOp.operand1.displacement);
            }
            break;
        }

        case X86::JNB_JAE: {
            if(!(state.flags & X86::CpuState::FLAGS::CF)) { FUNCS::executeJmp(&state, decOp.operand1.displacement);}
            break;
        }

        case X86::JNBE_JA: {
            if(!(state.flags & X86::CpuState::FLAGS::CF || state.flags & X86::CpuState::FLAGS::ZF)) { FUNCS::executeJmp(&state, decOp.operand1.displacement);}
            break;
        }

        case X86::JNP_JPO: {
            if(!(state.flags & X86::CpuState::FLAGS::PF)) { FUNCS::executeJmp(&state, decOp.operand1.displacement);}
            break;
        }

        case X86::JNO: {
            if(!(state.flags & X86::CpuState::FLAGS::OF)) { FUNCS::executeJmp(&state, decOp.operand1.displacement);}
            break;
        }

        case X86::JNS: {
            if(!(state.flags & X86::CpuState::FLAGS::SF)) { FUNCS::executeJmp(&state, decOp.operand1.displacement);}
            break;
        }

        case X86::LOOP: {
            FUNCS::executeJmp(&state, decOp.operand1.displacement);
            break;
        }

        case X86::LOOPZ_LOOPE: {
            if(state.flags & X86::CpuState::FLAGS::ZF) { FUNCS::executeJmp(&state, decOp.operand1.displacement);}
            break;
        }

        case X86::LOOPNZ_LOOPNE: {
            if(!(state.flags & X86::CpuState::FLAGS::ZF)) { FUNCS::executeJmp(&state, decOp.operand1.displacement);}
            break;
        }

        case X86::JCXZ: {
            if(state.regs.cx == 0) { FUNCS::executeJmp(&state, decOp.operand1.displacement);}
            break;
        }

        default: {
            printf("ERROR: Executing op %s not yet implemented!", X86::opName(decOp.op));
        }
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
        ";\tdi: 0x%04x (%d)\n"
        ";\n"
        ";\tcs: 0x%04x (%d)\n"
        ";\tds: 0x%04x (%d)\n"
        ";\tss: 0x%04x (%d)\n"
        ";\tes: 0x%04x (%d)\n;"
        "\n"
        ";\tip: 0x%04x (%d)\n"
        ";\n"
        ";\tflags: ",
        state.regs.ax, state.regs.ax,
        state.regs.bx, state.regs.bx,
        state.regs.cx, state.regs.cx,
        state.regs.dx, state.regs.dx,
        state.regs.sp, state.regs.sp,
        state.regs.bp, state.regs.bp,
        state.regs.si, state.regs.si,
        state.regs.di, state.regs.di,
        state.regs.cs, state.regs.cs,
        state.regs.ds, state.regs.ds,
        state.regs.ss, state.regs.ss,
        state.regs.es, state.regs.es,
        state.regs.ip, state.regs.ip);
    X86::CpuState::printFlags(state.flags);
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
        if(execute) { 
            cpuState = executeOp(op, cpuState);
            currentByte = inputFileBuffer + cpuState.regs.ip;
        } else {
            currentByte += op.sizeInBytes;
        }
        printf("\n");
    }
    if(execute) { printFinalRegisters(cpuState); }
    free(inputFileBuffer);
}