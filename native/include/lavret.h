#ifndef LAVRET_H
#define LAVRET_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define LAVRET_ABI_VERSION 1u

typedef uint32_t LavretStatus;

#define LAVRET_STATUS_OK 0u
#define LAVRET_STATUS_INVALID_ARGUMENT 1u
#define LAVRET_STATUS_WRONG_THREAD 2u
#define LAVRET_STATUS_INVALID_HANDLE 3u
#define LAVRET_STATUS_STALE_HANDLE 4u
#define LAVRET_STATUS_FOREIGN_HANDLE 5u
#define LAVRET_STATUS_CROSS_DOCUMENT_HANDLE 6u
#define LAVRET_STATUS_DESTROYED_HANDLE 7u
#define LAVRET_STATUS_INVALID_MUTATION 8u
#define LAVRET_STATUS_UNSUPPORTED_RESOURCE_SCHEME 9u
#define LAVRET_STATUS_RESOURCE_LIMIT 10u
#define LAVRET_STATUS_RESOURCE_DECODE 11u
#define LAVRET_STATUS_UNRESOLVED_DOCUMENT 12u
#define LAVRET_STATUS_DOCUMENT_FAILURE 13u
#define LAVRET_STATUS_ADAPTER_UNAVAILABLE 14u
#define LAVRET_STATUS_DEVICE_CREATION 15u
#define LAVRET_STATUS_GPU_EXECUTION 16u
#define LAVRET_STATUS_GPU_READBACK 17u
#define LAVRET_STATUS_DEVICE_LOST 18u
#define LAVRET_STATUS_UNSUPPORTED_OPERATION 19u
#define LAVRET_STATUS_PANIC 20u

#define LAVRET_POWER_LOW 0u
#define LAVRET_POWER_HIGH 1u

typedef struct LavretCore LavretCore;

typedef struct LavretBytes {
    uint8_t *data;
    size_t length;
} LavretBytes;

typedef struct LavretCoreOptions {
    uint32_t power_preference;
} LavretCoreOptions;

uint32_t lavret_abi_version(void);
LavretBytes lavret_last_error_message(void);
void lavret_bytes_free(LavretBytes bytes);
LavretStatus lavret_core_new(LavretCoreOptions options, LavretCore **output);
LavretStatus lavret_core_free(LavretCore *core);

#ifdef __cplusplus
}
#endif

#endif
