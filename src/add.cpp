u32 data[] = {
	1,
	2,
	3,
	4
};

u32 SingleSSE(u32 Count, u32 *Input)
{
	__m128i Sum = _mm_setzero_si128();
	for(u32 Index = 0; Index < Count; Index += 4)
	{
		Sum = _mm_add_epi32(Sum, _mm_load_si128((__m128i *)&Input[Index]));
	}

	Sum = _mm_hadd_epi32(Sum, Sum);
	Sum = _mm_hadd_epi32(Sum, Sum);
	
	return _mm_cvtsi128_si32(Sum);
}