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

X86::RegMem decodeRegMem(u8 rmByte, X86::WIDTH width) {
    u8 mod = rmByte >> 6;
    u8 rm = rmByte & 0b111;

    X86::RegMem result{};

    // register to register
    if(mod == 0b11) {
        result.reg = decodeReg(rm, width);
        result.flags = setFlags(result.flags, X86::RM_FLAGS_REG);
        return result;
    }

    // direct access special case
    if(rm == 0b110 && mod == 0b00) {
        result.flags = X86::RM_FLAGS_DISPLACE_16BIT;
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
        result.flags = setFlags(result.flags, X86::RM_FLAGS_DISPLACE_8BIT);
    } else if(mod == 0b10) {
        result.flags = setFlags(result.flags, X86::RM_FLAGS_DISPLACE_16BIT);
    }

    return result;
}

X86::DecodedOp decodeOp(u8* bytes) {
    u8 bytesRead = 0;
    u8 firstByte = bytes[bytesRead++];
    X86::OpMetadata firstByteMetadata = X86::opMetadata[firstByte];
    assert(firstByteMetadata.op != 0 && "ERROR: unsupported instruction");

    // NOTE: Dest operand can be Reg or Mem. Src can be Reg, Mem or Imm.
    X86::Operand dstOperand{}, srcOperand{};

    X86::WIDTH width = (firstByteMetadata.flags & X86::WIDTH_WORD) ? X86::WIDTH::WORD : X86::WIDTH::BYTE;
    bool wide = width == X86::WIDTH::WORD;
    bool regIsDst = firstByteMetadata.flags & X86::REG_IS_DST;
    
    bool containsReg = firstByteMetadata.flags & (X86::REG_BYTE1 | X86::REG_BYTE2);
    bool containsRM = firstByteMetadata.flags & X86::MOD_RM;
    u8 rmByte = containsRM ? bytes[bytesRead++] : 0;

    if(containsReg) {
        X86::Operand* regOperand = regIsDst ? &dstOperand : &srcOperand;
        u8 regCode = (firstByteMetadata.flags & X86::REG_BYTE1 ? firstByte : (rmByte >> 3)) & 0b111;
        regOperand->reg = decodeReg(regCode, width);
        regOperand->flags = X86::OPERAND_REG | (wide ? X86::OPERAND_WIDE : 0);
    }

    if(containsRM) {
        if(firstByteMetadata.flags & X86::ADDTL_OP_CODE) {
            assert((firstByteMetadata.flags & X86::REG_BYTE2) == 0);
            assert(firstByteMetadata.op == X86::PARTIAL_OP);
            assert(firstByteMetadata.extOps);
            firstByteMetadata.op = firstByteMetadata.extOps[(rmByte >> 3) & 0b111];
        }

        X86::RegMem regMem = decodeRegMem(rmByte, width);

        X86::Operand* rmOperand = regIsDst ? &srcOperand : &dstOperand;
        rmOperand->flags = setFlags(rmOperand->flags, (wide ? X86::OPERAND_WIDE : 0));

        if(regMem.isReg()) {
            rmOperand->reg = regMem.reg;
            rmOperand->flags = setFlags(rmOperand->flags, X86::OPERAND_REG);
        } else {
            assert(regMem.isMem());

            if(regMem.flags & X86::RM_FLAGS_REG1_EFF_ADDR) { // mem = [reg + ?]
                rmOperand->memReg1 = regMem.memReg1;
                rmOperand->flags = setFlags(rmOperand->flags, X86::OPERAND_MEM_REG1);
            }

            if(regMem.flags & X86::RM_FLAGS_REG2_EFF_ADDR) { // mem = [memReg1 + memReg2 + ?]
                rmOperand->memReg2 = regMem.memReg2;
                rmOperand->flags = setFlags(rmOperand->flags, X86::OPERAND_MEM_REG2);
            }

            if(regMem.flags & X86::RM_FLAGS_DISPLACE_8BIT) {
                rmOperand->displacement = *(s8*)&bytes[bytesRead++];
                rmOperand->flags = setFlags(rmOperand->flags, X86::OPERAND_DISPLACEMENT);
            } else if(regMem.flags & X86::RM_FLAGS_DISPLACE_16BIT) {
                rmOperand->displacement = *(s16*)&bytes[bytesRead]; bytesRead += 2;
                rmOperand->flags = setFlags(rmOperand->flags, X86::OPERAND_DISPLACEMENT);
            }
        }
    }

    if(firstByteMetadata.flags & X86::IMM) {
        srcOperand.flags = X86::OPERAND_IMM;
        if(!wide || (firstByteMetadata.flags & X86::SIGN_EXT)) {
            srcOperand.immediate = *(s8*)&bytes[bytesRead++];
        } else {
            srcOperand.immediate = *(s16*)&bytes[bytesRead]; bytesRead += 2;
        }
    }

    if(firstByteMetadata.flags & X86::MEM) {
        X86::Operand* memOperand = regIsDst ? &srcOperand : &dstOperand;
        memOperand->flags = X86::OPERAND_DISPLACEMENT;
        memOperand->displacement = *(s16*)&bytes[bytesRead]; bytesRead += 2;
    }

    if(firstByteMetadata.flags & X86::INC_IP_8BIT) {
        dstOperand.displacement = *(s8*)&bytes[bytesRead++];
        dstOperand.flags = X86::OPERAND_DISPLACEMENT;
        srcOperand.flags = X86::OPERAND_NO_OPERAND;
    }

    X86::DecodedOp result{};
    result.op = firstByteMetadata.op;
    result.operandDst = dstOperand;
    result.operandSrc = srcOperand;
    result.sizeInBytes = bytesRead;
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
            if(prevOperand->isMem()) {
                bool wide = prevOperand->flags & X86::OPERAND_WIDE;
                printf("%s ", wide ? "word" : "byte");
            }
            printf("%d", operand.immediate);
        }
    };

    if(op.operandSrc.flags & X86::OPERAND_NO_OPERAND) { // assume some kind of jmp
        printf("$%+d", op.operandDst.displacement + 2);
    } else { 
        writeAndPrintOperand(op.operandDst, nullptr);
        printf(", ");
        writeAndPrintOperand(op.operandSrc, &op.operandDst);
    }
}

void executeOp(X86::DecodedOp decOp, X86::CpuState* state) {

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

        static u32 calcMemAddr(X86::CpuState* state, X86::Operand memOp) {
            assert(memOp.isMem());
            u32 memAddr = memOp.displacement;
            if(memOp.flags | X86::OPERAND_MEM_REG1) {
                memAddr += state->regVal(memOp.memReg1);
                if(memOp.flags | X86::OPERAND_MEM_REG2) {
                    memAddr += state->regVal(memOp.memReg2);
                }
            }
            return memAddr;
        }

        static u16 calcOperandVal(X86::CpuState* state, X86::Operand operand) {
            if(operand.isReg()) {
                return state->regVal(operand.reg);
            } else if(operand.isMem()) {
                u32 memAddr = calcMemAddr(state, operand);
                if(operand.flags & X86::OPERAND_WIDE) {
                    return state->memDataVal16(memAddr);
                } else {
                    return state->mem.data[memAddr];
                }
            } else {
                assert(operand.isImm());
                return operand.immediate;
            }
        }

        static void executeAdd(X86::CpuState* state, X86::Operand dest, u16 val) {
            u16 oldFlags = state->flags;
            state->clearFlags();

            bool wide = dest.flags & X86::OPERAND_WIDE;

            assert(dest.isReg());

            if(wide) {
                u16 prevVal = state->regVal(dest.reg);
                u16 newVal = prevVal + val;
                state->regSet(dest.reg, newVal);
                printRegChange(state, dest.reg, prevVal);
            } else {
                u8 prevVal = (u8)state->regVal(dest.reg);
                u8 newVal = prevVal + (u8)val;
                state->regSet(dest.reg, newVal);
                printRegChange(state, dest.reg, prevVal);
            }

            updateSZPFlagsRegister(state, state->regVal(dest.reg));
            printFlagsChange(oldFlags, state->flags);
        }

        static void executeAdd(X86::CpuState* state, X86::Operand dest, X86::Operand src) {
            executeAdd(state, dest, calcOperandVal(state, src));
        }

        static void executeSub(X86::CpuState* state, X86::Operand dest, X86::Operand src) {
            executeAdd(state, dest, (u16)0 - calcOperandVal(state, src));
        }

        static void executeCmp(X86::CpuState* state, u16 val1, u16 val2) {
            u16 oldFlags = state->flags;
            state->clearFlags();
            u16 cmpRes = val1 - val2;
            updateSZPFlagsRegister(state, cmpRes);
            printFlagsChange(oldFlags, state->flags);
        }

        static void executeCmp(X86::CpuState* state, X86::Operand dest, X86::Operand src) {
            executeCmp(state, calcOperandVal(state, dest), calcOperandVal(state, src));
        }

        static void executeMov(X86::CpuState* state, X86::Operand dest, X86::Operand src) {
            bool wide = dest.flags & X86::OPERAND_WIDE;
            if(dest.isReg()) {
                if(src.isReg()) {
                    state->regSet(dest.reg, state->regVal(src.reg));
                } else if(src.isMem()) {
                    u32 memAddr = calcMemAddr(state, src);
                    state->regSet(dest.reg, wide ? state->memDataVal16(memAddr) : state->mem.data[memAddr]);
                } else if(src.isImm()) {
                    state->regSet(dest.reg, src.immediate);
                }
            } else if(dest.isMem()) {
                u32 memAddr = calcMemAddr(state, dest);

                if(src.isReg()) {
                    u16 val = state->regVal(src.reg);
                    if(wide) {
                        state->memDataSet16(memAddr, val);
                    } else {
                        state->mem.data[memAddr] = (u8)val;
                    }
                } else if(src.isImm()) {
                    if(wide) {
                        state->memDataSet16(memAddr, src.immediate);
                    } else {
                        state->mem.data[memAddr] = (u8)src.immediate;
                    }
                } else if(src.isMem()) {
                    assert(false && "ERROR: Mem to mem mov is not a part of the x86 ISA.");
                }
            }
        }

        static void executeJmp(X86::CpuState* state, s16 displacement) {
            state->regs.ip += displacement;
        }

        static void executeDec(X86::CpuState* state, X86::REG reg) {
            u16 val = state->regVal(reg);
            state->regSet(reg, val - 1);
        }
    };

    state->regs.ip += decOp.sizeInBytes;

    printf(" ; ");

    // Empty switch statement for all op.op cases
    
    switch(decOp.op) {
        case X86::MOV_IMM_TO_REG:
        case X86::MOV_RM_TO_FROM_REG:
        case X86::MOV_IMM_TO_RM:
        case X86::MOV_MEM_TO_ACC:
        case X86::MOV_ACC_TO_MEM: {
            FUNCS::executeMov(state, decOp.operandDst, decOp.operandSrc);
            break;
        }

        case X86::ADD_IMM_TO_ACC:
        case X86::ADD_IMM_TO_RM:
        case X86::ADD_RM_TO_FROM_REG: {
            FUNCS::executeAdd(state, decOp.operandDst, decOp.operandSrc);
            break;
        }

        case X86::SUB_IMM_FROM_ACC:
        case X86::SUB_IMM_FROM_RM:
        case X86::SUB_RM_TO_FROM_REG: {
            FUNCS::executeSub(state, decOp.operandDst, decOp.operandSrc);
            break;
        }

        case X86::CMP_IMM_AND_ACC:
        case X86::CMP_IMM_AND_RM:
        case X86::CMP_RM_AND_REG: {
            FUNCS::executeCmp(state, decOp.operandDst, decOp.operandSrc);
            break;
        }

        case X86::JE_JZ: {
            if(state->flags & X86::CpuState::FLAGS::ZF) { FUNCS::executeJmp(state, decOp.operandDst.displacement);}
            break;
        }

        case X86::JL_JNGE: {
            if(state->flags & X86::CpuState::FLAGS::SF) { FUNCS::executeJmp(state, decOp.operandDst.displacement);}
            break;
        }

        case X86::JLE_JNG: {
            if(state->flags & (X86::CpuState::FLAGS::ZF | X86::CpuState::FLAGS::SF)) { FUNCS::executeJmp(state, decOp.operandDst.displacement);}
            break;
        }

        case X86::JB_JNAE: {
            if(state->flags & X86::CpuState::FLAGS::CF) { FUNCS::executeJmp(state, decOp.operandDst.displacement);}
            break;
        }

        case X86::JBE_JNA: {
            if(state->flags & X86::CpuState::FLAGS::CF || state->flags & X86::CpuState::FLAGS::ZF) { FUNCS::executeJmp(state, decOp.operandDst.displacement); }
            break;
        }

        case X86::JP_JPE: {
            if(state->flags & X86::CpuState::FLAGS::PF) { FUNCS::executeJmp(state, decOp.operandDst.displacement);}
            break;
        }

        case X86::JO: {
            if(state->flags & X86::CpuState::FLAGS::OF) { FUNCS::executeJmp(state, decOp.operandDst.displacement);}
            break;
        }

        case X86::JS: {
            if(state->flags & X86::CpuState::FLAGS::SF) { FUNCS::executeJmp(state, decOp.operandDst.displacement);}
            break;
        }

        case X86::JNE_JNZ: {
            if(!(state->flags & X86::CpuState::FLAGS::ZF)) { FUNCS::executeJmp(state, decOp.operandDst.displacement);}
            break;
        }

        case X86::JNL_JGE: {
            if(XOR(state->flags & X86::CpuState::FLAGS::SF, state->flags & X86::CpuState::FLAGS::OF)) { 
                FUNCS::executeJmp(state, decOp.operandDst.displacement);
            }
            break;
        }

        case X86::JNLE_JG: {
            if(!(XOR(state->flags & X86::CpuState::FLAGS::SF, state->flags & X86::CpuState::FLAGS::OF) || (state->flags & X86::CpuState::FLAGS::ZF))) { 
                FUNCS::executeJmp(state, decOp.operandDst.displacement);
            }
            break;
        }

        case X86::JNB_JAE: {
            if(!(state->flags & X86::CpuState::FLAGS::CF)) { FUNCS::executeJmp(state, decOp.operandDst.displacement);}
            break;
        }

        case X86::JNBE_JA: {
            if(!(state->flags & X86::CpuState::FLAGS::CF || state->flags & X86::CpuState::FLAGS::ZF)) { FUNCS::executeJmp(state, decOp.operandDst.displacement);}
            break;
        }

        case X86::JNP_JPO: {
            if(!(state->flags & X86::CpuState::FLAGS::PF)) { FUNCS::executeJmp(state, decOp.operandDst.displacement);}
            break;
        }

        case X86::JNO: {
            if(!(state->flags & X86::CpuState::FLAGS::OF)) { FUNCS::executeJmp(state, decOp.operandDst.displacement);}
            break;
        }

        case X86::JNS: {
            if(!(state->flags & X86::CpuState::FLAGS::SF)) { FUNCS::executeJmp(state, decOp.operandDst.displacement);}
            break;
        }

        case X86::LOOP: {
            FUNCS::executeDec(state, X86::CX);
            if(state->regs.cx != 0) { FUNCS::executeJmp(state, decOp.operandDst.displacement);}
            break;
        }

        case X86::LOOPZ_LOOPE: {
            FUNCS::executeDec(state, X86::CX);
            if(state->regs.cx == 0 && state->flags & X86::CpuState::FLAGS::ZF) { 
                FUNCS::executeJmp(state, decOp.operandDst.displacement);
            }
            break;
        }

        case X86::LOOPNZ_LOOPNE: {
            FUNCS::executeDec(state, X86::CX);
            if(state->regs.cx != 0 && !(state->flags & X86::CpuState::FLAGS::ZF)) { 
                FUNCS::executeJmp(state, decOp.operandDst.displacement);
            }
            break;
        }

        case X86::JCXZ: {
            if(state->regs.cx == 0) { FUNCS::executeJmp(state, decOp.operandDst.displacement);}
            break;
        }

        default: {
            printf("ERROR: Executing op %s not yet implemented!", X86::opName(decOp.op));
        }
    }
}

void printFinalRegisters(const X86::CpuState& state) {
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

void dumpMemory(const X86::CpuState& state) {
    FILE* dumpFile = fopen("sim8086_dump.data", "wb");
    fwrite(state.memory, sizeof(state.memory), 1, dumpFile);
    fclose(dumpFile);
}

void decode8086Binary(const char* asmFilePath, DecodeOptions options) {
    long fileLen;
    X86::CpuState* cpuState = new X86::CpuState{};

    FILE* inputFile = fopen(asmFilePath, "rb");
    if(!inputFile) {
        printf("ERROR: Could not find file [%s]\nABORTING", asmFilePath);
        return;
    }
    fseek(inputFile, 0, SEEK_END);
    fileLen = ftell(inputFile);
    rewind(inputFile);

    assert(fileLen < sizeof(cpuState->mem.code));
    fread(cpuState->mem.code, fileLen, 1, inputFile);
    fclose(inputFile);
    inputFile = nullptr;

    printf(OUTPUT_FILE_HEADER);
    
    u8* lastByte = cpuState->mem.code + fileLen;
    u8* currentByte = cpuState->mem.code;
    while(currentByte < lastByte) {
        X86::DecodedOp op = decodeOp(currentByte); 
        printOp(op);
        if(options.execute) { 
            executeOp(op, cpuState);
            currentByte = cpuState->mem.code + cpuState->regs.ip;
        } else {
            currentByte += op.sizeInBytes;
        }
        printf("\n");
    }
    if(options.execute) { printFinalRegisters(*cpuState); }
    if(options.dump) { dumpMemory(*cpuState); }
    delete cpuState;
}