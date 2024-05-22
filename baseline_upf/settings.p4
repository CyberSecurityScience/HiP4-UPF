
#define PDR_ID_BITS 24
#define QER_ID_BITS 16
#define SHORT_TEID_BITS 18
#define COMPRESSED_QFI_BITS 2

#define NUM_QFI 5
#define NUM_PORT 5


#define MAX_PDR_PER_UE 2
#define NUM_UES 1638

#define PDR_TABLE_SIZE ( \
    MAX_PDR_PER_UE * NUM_UES \
)

#define SIMPLE_PDR_TABLE_SIZE  81920

#define TABLE_SIZE_IPV4_LPM 512
#define TABLE_SIZE_IPV4_EXACT 2048

#define PORT_BUFFER 134
#define PORT_CPU 64

#define N6_PORT_MAPPING 9w1: parse_overlay;
#define UPF_MAC 48w0x000c2980b582

#define CPU_HEADER_MAGIC 114

#define TABLE_SIZE_ACCOUNTING (PDR_TABLE_SIZE + SIMPLE_PDR_TABLE_SIZE * 2)