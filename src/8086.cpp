#define OUTPUT_FILE_HEADER "; Instruction decoding on the 8086 Homework by Connor Haskins\n\nbits 16\n"

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
        result.flags = turnOnFlags(result.flags, X86::RM_FLAGS_REG);
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
        result.flags = turnOnFlags(result.flags, X86::RM_FLAGS_REG1_EFF_ADDR | X86::RM_FLAGS_REG2_EFF_ADDR);
    } else {
        result.memReg1 = effAddrRegs5to7[rm - 4];
        result.flags = turnOnFlags(result.flags, X86::RM_FLAGS_REG1_EFF_ADDR);
    }
    
    if(mod == 0b01) {
        result.flags = turnOnFlags(result.flags, X86::RM_FLAGS_HAS_DISPLACE_8BIT);
    } else if(mod == 0b10) {
        result.flags = turnOnFlags(result.flags, X86::RM_FLAGS_HAS_DISPLACE_16BIT);
    }

    return result;
}

X86::DecodedOp decodeOp(u8* bytes) {

    X86::DecodedOp result{};
    u8 byte1 = *bytes++;

    X86::OpMetadata firstByteMetadata = X86::opMetadata[byte1];
    assert(firstByteMetadata.op != 0);

    result.op = firstByteMetadata.op;
    bool regIsDest = firstByteMetadata.flags & X86::REG_IS_DST;
    X86::WIDTH regWidth = X86::WIDTH((firstByteMetadata.flags & X86::WIDTH_WORD) > 0);

    if(firstByteMetadata.flags & X86::ACC) {
        X86::REG reg = regWidth == X86::WORD ? X86::AX : X86::AL;
        if(regIsDest) { // TODO: factor out
            result.operand1.reg = reg;
            result.operand1.flags = X86::OPERAND_REG;
        } else {
            result.operand2.reg = reg;
            result.operand2.flags = X86::OPERAND_REG;
        }
    }

    if(firstByteMetadata.flags & X86::REG_BYTE1) {
        X86::REG reg = decodeReg(byte1 & 0b111, regWidth);
        if(regIsDest) { // TODO: factor out
            result.operand1.reg = reg;
            result.operand1.flags = X86::OPERAND_REG;
        } else {
            result.operand2.reg = reg;
            result.operand2.flags = X86::OPERAND_REG;
        }
    }
    
    if(firstByteMetadata.flags & X86::MOD_RM) {
        u8 byte2 = *bytes++; // mod reg rm

        if(firstByteMetadata.flags & X86::REG_BYTE2) {
            X86::REG reg = decodeReg((byte2 & 0b00111000) >> 3, regWidth);
            if(regIsDest) { // TODO: factor out
                result.operand1.reg = reg;
                result.operand1.flags = X86::OPERAND_REG;
            } else {
                result.operand2.reg = reg;
                result.operand2.flags = X86::OPERAND_REG;
            }
        } 
        
        if(firstByteMetadata.flags & X86::ADDTL_OP_CODE) {
            assert((firstByteMetadata.flags & X86::REG_BYTE2) == 0);
            assert(result.op == X86::PARTIAL_OP);
            assert(firstByteMetadata.extOps);
            result.op = firstByteMetadata.extOps[(byte2 & 0b00111000) >> 3];
        }

        X86::RegMem regMem = decodeRegMem(byte2 >> 6, byte2 & 0b111, regWidth);

        X86::Operand* rmOperand;
        if(regIsDest) {
            rmOperand = &result.operand2;
        } else {
            rmOperand = &result.operand1;
        }

        if(regMem.isReg()) {
            rmOperand->reg = regMem.reg;
            rmOperand->flags = turnOnFlags(rmOperand->flags, X86::OPERAND_REG);
        } else {
            if(regMem.flags & X86::RM_FLAGS_REG1_EFF_ADDR) { // mem = [reg + ?]
                rmOperand->memReg1 = regMem.memReg1;
                rmOperand->flags = turnOnFlags(rmOperand->flags, X86::OPERAND_MEM_REG1);
            }

            if(regMem.flags & X86::RM_FLAGS_REG2_EFF_ADDR) { // mem = [memReg1 + memReg2 + ?]
                rmOperand->memReg2 = regMem.memReg2;
                rmOperand->flags = turnOnFlags(rmOperand->flags, X86::OPERAND_MEM_REG2);
            }

            if(regMem.flags & X86::RM_FLAGS_HAS_DISPLACE_8BIT) {
                rmOperand->displacement = *(s8*)bytes++;
                rmOperand->flags = turnOnFlags(rmOperand->flags, X86::OPERAND_DISPLACEMENT);
            } else if(regMem.flags & X86::RM_FLAGS_HAS_DISPLACE_16BIT) {
                rmOperand->displacement = *(s16*)bytes; bytes += 2;
                rmOperand->flags = turnOnFlags(rmOperand->flags, X86::OPERAND_DISPLACEMENT);
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
        X86::Operand* memOperand = regIsDest ? &result.operand2 : &result.operand1;
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
        X86::DecodedOp op = decodeOp(currentByte); 
        currentByte = op.nextByte;

        printf("\n");
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
            op.operand1.displacement >= 0 ? printf("$+%d", op.operand1.displacement) : printf("$%d", op.operand1.displacement);
        } else { 
            writeAndPrintOperand(op.operand1, nullptr);
            printf(", ");
            writeAndPrintOperand(op.operand2, &op.operand1);
        }
    }
    free(inputFileBuffer);
}