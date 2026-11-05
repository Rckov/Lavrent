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

#define LAVRET_COLOR_LIGHT 0u
#define LAVRET_COLOR_DARK 1u

#define LAVRET_RESOURCE_AUTHOR_STYLESHEET 0u
#define LAVRET_RESOURCE_RASTER_IMAGE 1u
#define LAVRET_RESOURCE_SVG_DOCUMENT 2u
#define LAVRET_RESOURCE_FONT 3u

typedef struct LavretCore LavretCore;

typedef struct LavretBytes {
    uint8_t *data;
    size_t length;
} LavretBytes;

typedef struct LavretCoreOptions {
    uint32_t power_preference;
} LavretCoreOptions;

typedef struct LavretStringView {
    const uint8_t *data;
    size_t length;
} LavretStringView;

typedef struct LavretDocumentSource {
    LavretStringView html;
    const LavretStringView *author_css;
    size_t author_css_length;
    LavretStringView base_resource_uri;
} LavretDocumentSource;

typedef struct LavretViewport {
    double width;
    double height;
    double scale_factor;
    uint32_t color_scheme;
} LavretViewport;

typedef struct LavretPoint {
    double x;
    double y;
} LavretPoint;

typedef struct LavretRect {
    double x;
    double y;
    double width;
    double height;
} LavretRect;

typedef struct LavretHandleArray {
    uint64_t *data;
    size_t length;
} LavretHandleArray;

typedef struct LavretResourceSource {
    LavretStringView uri;
    LavretStringView media_type;
    uint32_t kind;
    const uint8_t *data;
    size_t length;
} LavretResourceSource;

typedef struct LavretRenderResult {
    uint32_t width;
    uint32_t height;
} LavretRenderResult;

typedef struct LavretReadback {
    uint32_t width;
    uint32_t height;
    LavretBytes pixels;
} LavretReadback;

uint32_t lavret_abi_version(void);
LavretBytes lavret_last_error_message(void);
void lavret_bytes_free(LavretBytes bytes);
void lavret_handles_free(LavretHandleArray handles);
LavretStatus lavret_core_new(LavretCoreOptions options, LavretCore **output);
LavretStatus lavret_core_free(LavretCore *core);
LavretStatus lavret_document_create(LavretCore *core, LavretDocumentSource source, LavretViewport viewport, uint64_t *output);
LavretStatus lavret_document_destroy(LavretCore *core, uint64_t document);
LavretStatus lavret_document_replace_source(LavretCore *core, uint64_t document, LavretDocumentSource source);
LavretStatus lavret_document_find_element_by_id(LavretCore *core, uint64_t document, LavretStringView id, uint64_t *output);
LavretStatus lavret_node_create_element(LavretCore *core, uint64_t document, LavretStringView name, uint64_t *output);
LavretStatus lavret_node_create_text(LavretCore *core, uint64_t document, LavretStringView text, uint64_t *output);
LavretStatus lavret_node_append(LavretCore *core, uint64_t document, uint64_t parent, uint64_t child);
LavretStatus lavret_node_insert_before(LavretCore *core, uint64_t document, uint64_t anchor, uint64_t node);
LavretStatus lavret_node_move(LavretCore *core, uint64_t document, uint64_t node, uint64_t new_parent, uint64_t before);
LavretStatus lavret_node_remove(LavretCore *core, uint64_t document, uint64_t node);
LavretStatus lavret_node_set_attribute(LavretCore *core, uint64_t document, uint64_t node, LavretStringView name, LavretStringView value);
LavretStatus lavret_node_remove_attribute(LavretCore *core, uint64_t document, uint64_t node, LavretStringView name);
LavretStatus lavret_node_replace_text(LavretCore *core, uint64_t document, uint64_t node, LavretStringView text);
LavretStatus lavret_document_set_viewport(LavretCore *core, uint64_t document, LavretViewport viewport);
LavretStatus lavret_document_resolve(LavretCore *core, uint64_t document, uint64_t animation_time_nanoseconds);
LavretStatus lavret_node_border_box(LavretCore *core, uint64_t document, uint64_t node, LavretRect *output);
LavretStatus lavret_document_hit_test(LavretCore *core, uint64_t document, LavretPoint point, LavretHandleArray *output);
LavretStatus lavret_resource_register(LavretCore *core, LavretResourceSource source, uint64_t *output);
LavretStatus lavret_resource_replace(LavretCore *core, uint64_t resource, LavretResourceSource source);
LavretStatus lavret_resource_remove(LavretCore *core, uint64_t resource);
LavretStatus lavret_render_target_create(LavretCore *core, uint32_t width, uint32_t height, uint64_t *output);
LavretStatus lavret_render_target_resize(LavretCore *core, uint64_t target, uint32_t width, uint32_t height);
LavretStatus lavret_render_target_destroy(LavretCore *core, uint64_t target);
LavretStatus lavret_render(LavretCore *core, uint64_t document, uint64_t target, LavretRenderResult *output);
LavretStatus lavret_render_target_readback_rgba8(LavretCore *core, uint64_t target, LavretReadback *output);
void lavret_readback_free(LavretReadback readback);

#ifdef __cplusplus
}
#endif

#endif
