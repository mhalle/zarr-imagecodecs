"""Zarr v3 image codecs backed by Rust.

Provides ArrayBytesCodec implementations for image formats not
covered by zarr-python's built-in codecs: JPEG 2000, JPEG, JPEG XL,
PNG, WebP, AVIF, and TIFF.

Usage::

    import zarr
    import numpy as np
    from zarr_imagecodecs import Jpeg2k

    arr = zarr.create_array(
        store='test.zarr',
        shape=(512, 512, 3),
        chunks=(256, 256, 3),
        dtype='uint8',
        codecs=[Jpeg2k(reversible=True)],
        overwrite=True,
    )
    arr[:] = np.random.randint(0, 255, (512, 512, 3), dtype='uint8')
"""

from zarr_imagecodecs._codecs import (
    Avif,
    Jpeg,
    Jpeg2k,
    Jpegxl,
    Png,
    Tiff,
    Webp,
)

__all__ = [
    'Avif',
    'Jpeg',
    'Jpeg2k',
    'Jpegxl',
    'Png',
    'Tiff',
    'Webp',
]
