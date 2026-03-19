#include "ojph_shim.h"

#include "ojph_codestream.h"
#include "ojph_file.h"
#include "ojph_params.h"
#include "ojph_mem.h"

#include <cstring>
#include <cstdlib>

extern "C" {

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
) {
    try {
        ojph::codestream cs;
        ojph::mem_outfile outfile;

        // Configure image size
        ojph::param_siz siz = cs.access_siz();
        siz.set_image_extent(ojph::point(width, height));
        siz.set_num_components(num_comps);
        for (uint32_t c = 0; c < num_comps; c++)
            siz.set_component(c, ojph::point(1, 1), bit_depth, is_signed != 0);

        // Configure coding
        ojph::param_cod cod = cs.access_cod();
        cod.set_num_decomposition(num_decomps);
        cod.set_block_dims(64, 64);
        cod.set_reversible(reversible != 0);
        cod.set_color_transform(num_comps >= 3);

        if (!reversible && quant_step > 0.0f) {
            ojph::param_qcd qcd = cs.access_qcd();
            qcd.set_irrev_quant(quant_step);
        }

        // Set planar=false for interleaved component order
        cs.set_planar(num_comps < 2);

        // Open output and write headers
        outfile.open();
        cs.write_headers(&outfile);

        int bytes_per_sample = (bit_depth + 7) / 8;

        // Feed lines: the codestream tells us which component it wants next
        // via next_comp. We must supply lines in the order it requests.
        ojph::ui32 next_comp;
        ojph::line_buf *line = cs.exchange(NULL, next_comp);

        // We need to feed height * num_comps lines total
        // (one line per component per row)
        uint32_t cur_line[16] = {}; // current row per component

        for (uint32_t i = 0; i < height * num_comps; i++) {
            uint32_t c = next_comp;
            uint32_t y = cur_line[c];
            cur_line[c]++;

            ojph::si32 *dst = line->i32;
            for (uint32_t x = 0; x < width; x++) {
                size_t pixel_offset =
                    (size_t)y * width * num_comps * bytes_per_sample
                    + (size_t)x * num_comps * bytes_per_sample
                    + (size_t)c * bytes_per_sample;

                if (bytes_per_sample == 1) {
                    if (is_signed)
                        dst[x] = (ojph::si32)((int8_t)pixels[pixel_offset]);
                    else
                        dst[x] = (ojph::si32)pixels[pixel_offset];
                } else {
                    uint16_t val = pixels[pixel_offset]
                                 | ((uint16_t)pixels[pixel_offset + 1] << 8);
                    if (is_signed)
                        dst[x] = (ojph::si32)(int16_t)val;
                    else
                        dst[x] = (ojph::si32)val;
                }
            }
            line = cs.exchange(line, next_comp);
        }

        cs.flush();
        cs.close();

        // Copy output
        *out_size = outfile.get_used_size();
        *out_buf = (uint8_t *)malloc(*out_size);
        if (!*out_buf) return -1;
        memcpy(*out_buf, outfile.get_data(), *out_size);

        outfile.close();
        return 0;

    } catch (...) {
        return -1;
    }
}

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
) {
    try {
        ojph::codestream cs;
        ojph::mem_infile infile;

        infile.open(data, data_size);
        cs.read_headers(&infile);

        ojph::param_siz siz = cs.access_siz();
        *out_width = siz.get_recon_width(0);
        *out_height = siz.get_recon_height(0);
        *out_comps = siz.get_num_components();
        *out_depth = siz.get_bit_depth(0);
        *out_signed = siz.is_signed(0) ? 1 : 0;

        uint32_t w = *out_width;
        uint32_t h = *out_height;
        uint32_t nc = *out_comps;
        uint32_t bd = *out_depth;
        int bytes_per_sample = (bd + 7) / 8;

        cs.create();

        size_t total = (size_t)w * h * nc * bytes_per_sample;
        *out_buf = (uint8_t *)malloc(total);
        if (!*out_buf) return -1;
        *out_size = total;
        memset(*out_buf, 0, total);

        // Pull lines: the codestream tells us which component each line is for
        ojph::ui32 comp_num;
        uint32_t cur_line[16] = {};

        for (uint32_t i = 0; i < h * nc; i++) {
            ojph::line_buf *line = cs.pull(comp_num);
            uint32_t y = cur_line[comp_num];
            cur_line[comp_num]++;

            ojph::si32 *src = line->i32;

            for (uint32_t x = 0; x < w; x++) {
                size_t pixel_offset =
                    (size_t)y * w * nc * bytes_per_sample
                    + (size_t)x * nc * bytes_per_sample
                    + (size_t)comp_num * bytes_per_sample;

                ojph::si32 val = src[x];
                if (bytes_per_sample == 1) {
                    if (*out_signed)
                        (*out_buf)[pixel_offset] = (uint8_t)(int8_t)val;
                    else
                        (*out_buf)[pixel_offset] = (uint8_t)(val < 0 ? 0 : (val > 255 ? 255 : val));
                } else {
                    uint16_t uval;
                    if (*out_signed)
                        uval = (uint16_t)(int16_t)val;
                    else
                        uval = (uint16_t)(val < 0 ? 0 : (val > 65535 ? 65535 : val));
                    (*out_buf)[pixel_offset] = (uint8_t)(uval & 0xFF);
                    (*out_buf)[pixel_offset + 1] = (uint8_t)(uval >> 8);
                }
            }
        }

        cs.close();
        return 0;

    } catch (...) {
        return -1;
    }
}

void ojph_free(uint8_t *buf) {
    free(buf);
}

} // extern "C"
