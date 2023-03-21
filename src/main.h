#pragma once
#include <intrin.h>
#include <immintrin.h>
#include <smmintrin.h>
#include <wmmintrin.h>

#include <stdio.h>
#include <iostream>
#include <fstream>
#include <string>
#include <sstream>
#include <stdint.h>
#include <assert.h>

typedef int8_t s8;
typedef int16_t s16;
typedef int32_t s32;
typedef int64_t s64;

typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;

typedef float f32;
typedef double f64;

typedef s32 b32;

#define global_variable static
#define internal_func static // internal_func to translation unit
#define func_persist static
#define class_persist static

#define ArrayCount(Array) (sizeof(Array) / sizeof((Array)[0]))

// Out is used to label out function parameters
#define Out

#define InvalidCodePath assert(!"InvalidCodePath");

#include "platform_windows.h"

#include "util.h"
#include "add.cpp"
#include "8086.hpp"
#include "8086.cpp"