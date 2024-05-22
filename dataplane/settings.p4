
#define MA_ID_BITS 24 // global_PDR_ID bits (163k for 19bits, 169k for 24bits)
#define QER_ID_BITS 16
#define SHORT_TEID_BITS 18
#define COMPRESSED_QFI_BITS 2

#define NUM_QFI 5
#define NUM_PORT 5

#define TABLE_SIZE_UL_N6_SIMPLE_IPV4 (161792 - (2 * 1024))
#define TABLE_SIZE_UL_N6_COMPLEX_IPV4 ((3236) >> 1)

#define TABLE_SIZE_UL_N9_SIMPLE_IPV4 2 * 1024
#define TABLE_SIZE_UL_N9_COMPLEX_IPV4 ((3236) >> 1)


#define TABLE_SIZE_DL_N6_SIMPLE_IPV4 (161792 - (2 * 1024))
#define TABLE_SIZE_DL_N6_COMPLEX_IPV4 ((3236) >> 1)

#define TABLE_SIZE_DL_N9_SIMPLE_IPV4 (TABLE_SIZE_UL_N9_SIMPLE_IPV4 + TABLE_SIZE_UL_N9_COMPLEX_IPV4)

#define TABLE_SIZE_ACCOUNTING ( \
    TABLE_SIZE_DL_N6_SIMPLE_IPV4 + \
    TABLE_SIZE_DL_N6_COMPLEX_IPV4 + \
    TABLE_SIZE_DL_N9_SIMPLE_IPV4 + \
    TABLE_SIZE_UL_N6_SIMPLE_IPV4 + \
    TABLE_SIZE_UL_N6_COMPLEX_IPV4 + \
    TABLE_SIZE_UL_N9_SIMPLE_IPV4 + \
    TABLE_SIZE_UL_N9_COMPLEX_IPV4 \
)

#define TABLE_SIZE_ACCOUNTING_EXTRA <%VALUE%>

#define TABLE_SIZE_IPV4_EXACT 2048
#define TABLE_SIZE_IPV4_LPM 512

#define PORT_BUFFER 134
#define PORT_CPU 64

#define N6_PORT_MAPPING 9w1: parse_overlay;
#define UPF_MAC 48w0x000c2980b582

#define CPU_HEADER_MAGIC 114
