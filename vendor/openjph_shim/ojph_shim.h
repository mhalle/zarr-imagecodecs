#ifndef OJPH_SHIM_H
#define OJPH_SHIM_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Encode raw pixel data to HTJ2K.
 *
 * @param pixels      Raw pixel data (interleaved for multi-component)
 * @param width       Image width
 * @param height      Image height
 * @param num_comps   Number of components (1=gray, 3=RGB)
 * @param bit_depth   Bits per sample (8 or 16)
 * @param is_signed   Whether samples are signed
 * @param reversible  1=lossless, 0=lossy
 * @param quant_step  Quantization step for lossy (ignored if reversible)
 * @param num_decomps Number of wavelet decomposition levels (default 5)
 * @param out_buf     Output: pointer to encoded data (caller must free with ojph_free)
 * @param out_size    Output: size of encoded data
 * @return 0 on success, non-zero on error
 */
int ojph_encode(
    const uint8_t *pixels,
    uint32_t width,
    uint32_t height,
    uint32_t num_comps,
    uint32_t bit_depth,
    int is_signed,
    int reversible,
    float quant_step,
    uint32_t num_decomps,
    uint8_t **out_buf,
    size_t *out_size
);

/**
 * Decode HTJ2K data to raw pixels.
 *
 * @param data        HTJ2K encoded data
 * @param data_size   Size of encoded data
 * @param out_buf     Output: pointer to decoded pixel data (caller must free with ojph_free)
 * @param out_size    Output: size of decoded data
 * @param out_width   Output: image width
 * @param out_height  Output: image height
 * @param out_comps   Output: number of components
 * @param out_depth   Output: bit depth
 * @param out_signed  Output: whether signed
 * @return 0 on success, non-zero on error
 */
int ojph_decode(
    const uint8_t *data,
    size_t data_size,
    uint8_t **out_buf,
    size_t *out_size,
    uint32_t *out_width,
    uint32_t *out_height,
    uint32_t *out_comps,
    uint32_t *out_depth,
    int *out_signed
);

/**
 * Free a buffer returned by ojph_encode or ojph_decode.
 */
void ojph_free(uint8_t *buf);

#ifdef __cplusplus
}
#endif

#endif /* OJPH_SHIM_H */
