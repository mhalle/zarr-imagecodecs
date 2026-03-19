"""Zarr v3 codec classes wrapping Rust image encode/decode functions."""

from __future__ import annotations

import asyncio
from dataclasses import dataclass, fields
from typing import TYPE_CHECKING, Any, ClassVar

import numpy

from zarr.abc.codec import ArrayBytesCodec

if TYPE_CHECKING:
    from typing import Self

    from zarr.core.array_spec import ArraySpec
    from zarr.core.buffer import Buffer, NDBuffer

from zarr_imagecodecs._zarr_imagecodecs import (
    avif_decode,
    avif_encode,
    jpeg2k_decode,
    jpeg2k_encode,
    jpeg_decode,
    jpeg_encode,
    jpegxl_decode,
    jpegxl_encode,
    png_decode,
    png_encode,
    tiff_decode,
    tiff_encode,
    webp_decode,
    webp_encode,
    jpegls_decode,
    jpegls_encode,
    packbits_decode,
    packbits_encode,
)

from zarr.abc.codec import BytesBytesCodec


# --- Base class ---


@dataclass(frozen=True)
class _ImageCodec(ArrayBytesCodec):
    """Base class for image codecs."""

    _codec_name: ClassVar[str]
    is_fixed_size: ClassVar[bool] = False

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> Self:
        if isinstance(data, str):
            return cls()
        if 'name' in data and 'configuration' in data:
            config = dict(data['configuration'])
        else:
            config = {k: v for k, v in data.items() if k != 'name'}
        known = {f.name for f in fields(cls) if not f.name.startswith('_')}
        config = {k: v for k, v in config.items() if k in known}
        return cls(**config)

    def to_dict(self) -> dict[str, Any]:
        config = {}
        for f in fields(self):
            if f.name.startswith('_'):
                continue
            val = getattr(self, f.name)
            if val is not None:
                config[f.name] = val
        return {'name': self._codec_name, 'configuration': config}

    def compute_encoded_size(
        self, input_byte_length: int, chunk_spec: ArraySpec
    ) -> int:
        raise NotImplementedError


def _squeeze_image(arr: numpy.ndarray) -> numpy.ndarray:
    """Ensure array is at least 2D for image codecs."""
    return numpy.atleast_2d(numpy.squeeze(arr))


# --- Codec implementations ---


@dataclass(frozen=True)
class Jpeg2k(_ImageCodec):
    """JPEG 2000 codec for zarr v3.

    Parameters
    ----------
    level : float, optional
        Compression rate. 0 = lossless.
    reversible : bool
        Use reversible (lossless) wavelet transform.
    num_resolutions : int, optional
        Number of resolution levels.
    """

    _codec_name: ClassVar[str] = 'imagecodecs_jpeg2k'

    level: float | None = None
    reversible: bool = False
    num_resolutions: int | None = None

    async def _decode_single(
        self, chunk_data: Buffer, chunk_spec: ArraySpec
    ) -> NDBuffer:
        chunk_bytes = chunk_data.to_bytes()
        shape = list(chunk_spec.shape)

        def _decode() -> numpy.ndarray:
            return numpy.asarray(jpeg2k_decode(chunk_bytes, shape))

        out = await asyncio.to_thread(_decode)
        return chunk_spec.prototype.nd_buffer.from_ndarray_like(
            out.reshape(chunk_spec.shape)
        )

    async def _encode_single(
        self, chunk_data: NDBuffer, chunk_spec: ArraySpec
    ) -> Buffer | None:
        chunk_ndarray = numpy.ascontiguousarray(
            _squeeze_image(chunk_data.as_ndarray_like())
        )

        def _encode() -> bytes:
            return bytes(jpeg2k_encode(
                chunk_ndarray,
                level=self.level,
                reversible=self.reversible,
                num_resolutions=self.num_resolutions,
            ))

        out = await asyncio.to_thread(_encode)
        return chunk_spec.prototype.buffer.from_bytes(out)


@dataclass(frozen=True)
class Jpeg(_ImageCodec):
    """JPEG codec for zarr v3.

    Parameters
    ----------
    quality : int
        Compression quality (1-100). Default 90.
    """

    _codec_name: ClassVar[str] = 'imagecodecs_jpeg'

    quality: int = 90

    async def _decode_single(
        self, chunk_data: Buffer, chunk_spec: ArraySpec
    ) -> NDBuffer:
        chunk_bytes = chunk_data.to_bytes()
        shape = list(chunk_spec.shape)

        def _decode() -> numpy.ndarray:
            return numpy.asarray(jpeg_decode(chunk_bytes, shape))

        out = await asyncio.to_thread(_decode)
        return chunk_spec.prototype.nd_buffer.from_ndarray_like(
            out.reshape(chunk_spec.shape)
        )

    async def _encode_single(
        self, chunk_data: NDBuffer, chunk_spec: ArraySpec
    ) -> Buffer | None:
        chunk_ndarray = numpy.ascontiguousarray(
            _squeeze_image(chunk_data.as_ndarray_like())
        )

        def _encode() -> bytes:
            return bytes(jpeg_encode(chunk_ndarray, quality=self.quality))

        out = await asyncio.to_thread(_encode)
        return chunk_spec.prototype.buffer.from_bytes(out)


@dataclass(frozen=True)
class Jpegxl(_ImageCodec):
    """JPEG XL codec for zarr v3.

    Encoding uses pure-Rust zune-jpegxl (lossless only).
    Decoding uses pure-Rust jxl-oxide.

    Parameters
    ----------
    effort : int, optional
        Encoding effort (higher = slower + smaller).
    """

    _codec_name: ClassVar[str] = 'imagecodecs_jpegxl'

    effort: int | None = None

    async def _decode_single(
        self, chunk_data: Buffer, chunk_spec: ArraySpec
    ) -> NDBuffer:
        chunk_bytes = chunk_data.to_bytes()
        shape = list(chunk_spec.shape)

        def _decode() -> numpy.ndarray:
            return numpy.asarray(jpegxl_decode(chunk_bytes, shape))

        out = await asyncio.to_thread(_decode)
        return chunk_spec.prototype.nd_buffer.from_ndarray_like(
            out.reshape(chunk_spec.shape)
        )

    async def _encode_single(
        self, chunk_data: NDBuffer, chunk_spec: ArraySpec
    ) -> Buffer | None:
        chunk_ndarray = numpy.ascontiguousarray(
            _squeeze_image(chunk_data.as_ndarray_like())
        )

        def _encode() -> bytes:
            return bytes(jpegxl_encode(
                chunk_ndarray, effort=self.effort
            ))

        out = await asyncio.to_thread(_encode)
        return chunk_spec.prototype.buffer.from_bytes(out)


@dataclass(frozen=True)
class Png(_ImageCodec):
    """PNG codec for zarr v3.

    Parameters
    ----------
    level : int, optional
        Compression level (0=fast, 9=best).
    """

    _codec_name: ClassVar[str] = 'imagecodecs_png'

    level: int | None = None

    async def _decode_single(
        self, chunk_data: Buffer, chunk_spec: ArraySpec
    ) -> NDBuffer:
        chunk_bytes = chunk_data.to_bytes()
        shape = list(chunk_spec.shape)

        def _decode() -> numpy.ndarray:
            return numpy.asarray(png_decode(chunk_bytes, shape))

        out = await asyncio.to_thread(_decode)
        return chunk_spec.prototype.nd_buffer.from_ndarray_like(
            out.reshape(chunk_spec.shape)
        )

    async def _encode_single(
        self, chunk_data: NDBuffer, chunk_spec: ArraySpec
    ) -> Buffer | None:
        chunk_ndarray = numpy.ascontiguousarray(
            _squeeze_image(chunk_data.as_ndarray_like())
        )

        def _encode() -> bytes:
            return bytes(png_encode(chunk_ndarray, level=self.level))

        out = await asyncio.to_thread(_encode)
        return chunk_spec.prototype.buffer.from_bytes(out)


@dataclass(frozen=True)
class Webp(_ImageCodec):
    """WebP codec for zarr v3.

    Parameters
    ----------
    quality : float, optional
        Lossy quality (0-100). Default 75.
    lossless : bool
        Use lossless compression.
    """

    _codec_name: ClassVar[str] = 'imagecodecs_webp'

    quality: float | None = None
    lossless: bool = False

    async def _decode_single(
        self, chunk_data: Buffer, chunk_spec: ArraySpec
    ) -> NDBuffer:
        chunk_bytes = chunk_data.to_bytes()
        shape = list(chunk_spec.shape)

        def _decode() -> numpy.ndarray:
            return numpy.asarray(webp_decode(chunk_bytes, shape))

        out = await asyncio.to_thread(_decode)
        return chunk_spec.prototype.nd_buffer.from_ndarray_like(
            out.reshape(chunk_spec.shape)
        )

    async def _encode_single(
        self, chunk_data: NDBuffer, chunk_spec: ArraySpec
    ) -> Buffer | None:
        chunk_ndarray = numpy.ascontiguousarray(
            _squeeze_image(chunk_data.as_ndarray_like())
        )

        def _encode() -> bytes:
            return bytes(webp_encode(
                chunk_ndarray,
                quality=self.quality,
                lossless=self.lossless,
            ))

        out = await asyncio.to_thread(_encode)
        return chunk_spec.prototype.buffer.from_bytes(out)


@dataclass(frozen=True)
class Avif(_ImageCodec):
    """AVIF codec for zarr v3.

    Parameters
    ----------
    quality : float, optional
        Quality (0-100). Higher = better quality.
    speed : int, optional
        Encoding speed (1-10). Higher = faster but worse compression.
    """

    _codec_name: ClassVar[str] = 'imagecodecs_avif'

    quality: float | None = None
    speed: int | None = None

    async def _decode_single(
        self, chunk_data: Buffer, chunk_spec: ArraySpec
    ) -> NDBuffer:
        chunk_bytes = chunk_data.to_bytes()
        shape = list(chunk_spec.shape)

        def _decode() -> numpy.ndarray:
            return numpy.asarray(avif_decode(chunk_bytes, shape))

        out = await asyncio.to_thread(_decode)
        return chunk_spec.prototype.nd_buffer.from_ndarray_like(
            out.reshape(chunk_spec.shape)
        )

    async def _encode_single(
        self, chunk_data: NDBuffer, chunk_spec: ArraySpec
    ) -> Buffer | None:
        chunk_ndarray = numpy.ascontiguousarray(
            _squeeze_image(chunk_data.as_ndarray_like())
        )

        def _encode() -> bytes:
            return bytes(avif_encode(
                chunk_ndarray,
                quality=self.quality,
                speed=self.speed,
            ))

        out = await asyncio.to_thread(_encode)
        return chunk_spec.prototype.buffer.from_bytes(out)


@dataclass(frozen=True)
class Tiff(_ImageCodec):
    """TIFF codec for zarr v3.

    Parameters
    ----------
    compression : str, optional
        Compression method ('deflate', 'lzw', 'none').
    """

    _codec_name: ClassVar[str] = 'imagecodecs_tiff'

    compression: str | None = None

    async def _decode_single(
        self, chunk_data: Buffer, chunk_spec: ArraySpec
    ) -> NDBuffer:
        chunk_bytes = chunk_data.to_bytes()
        shape = list(chunk_spec.shape)

        def _decode() -> numpy.ndarray:
            return numpy.asarray(tiff_decode(chunk_bytes, shape))

        out = await asyncio.to_thread(_decode)
        return chunk_spec.prototype.nd_buffer.from_ndarray_like(
            out.reshape(chunk_spec.shape)
        )

    async def _encode_single(
        self, chunk_data: NDBuffer, chunk_spec: ArraySpec
    ) -> Buffer | None:
        chunk_ndarray = numpy.ascontiguousarray(
            _squeeze_image(chunk_data.as_ndarray_like())
        )

        def _encode() -> bytes:
            return bytes(tiff_encode(
                chunk_ndarray,
                compression=self.compression,
            ))

        out = await asyncio.to_thread(_encode)
        return chunk_spec.prototype.buffer.from_bytes(out)


@dataclass(frozen=True)
class Jpegls(_ImageCodec):
    """JPEG-LS codec for zarr v3.

    Parameters
    ----------
    near : int
        Near-lossless tolerance. 0 = lossless (default).
    """

    _codec_name: ClassVar[str] = 'imagecodecs_jpegls'

    near: int = 0

    async def _decode_single(
        self, chunk_data: Buffer, chunk_spec: ArraySpec
    ) -> NDBuffer:
        chunk_bytes = chunk_data.to_bytes()
        shape = list(chunk_spec.shape)

        def _decode() -> numpy.ndarray:
            return numpy.asarray(jpegls_decode(chunk_bytes, shape))

        out = await asyncio.to_thread(_decode)
        return chunk_spec.prototype.nd_buffer.from_ndarray_like(
            out.reshape(chunk_spec.shape)
        )

    async def _encode_single(
        self, chunk_data: NDBuffer, chunk_spec: ArraySpec
    ) -> Buffer | None:
        # JPEG-LS needs the original shape (H, W) or (H, W, C)
        chunk_ndarray = numpy.ascontiguousarray(
            chunk_data.as_ndarray_like()
        )

        def _encode() -> bytes:
            return bytes(jpegls_encode(chunk_ndarray, near=self.near))

        out = await asyncio.to_thread(_encode)
        return chunk_spec.prototype.buffer.from_bytes(out)


@dataclass(frozen=True)
class _BytesBytesBase(BytesBytesCodec):
    """Base class for bytes-to-bytes codecs."""

    _codec_name: ClassVar[str]
    is_fixed_size: ClassVar[bool] = False

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> Self:
        if isinstance(data, str):
            return cls()
        if 'name' in data and 'configuration' in data:
            config = dict(data['configuration'])
        else:
            config = {k: v for k, v in data.items() if k != 'name'}
        known = {f.name for f in fields(cls) if not f.name.startswith('_')}
        config = {k: v for k, v in config.items() if k in known}
        return cls(**config)

    def to_dict(self) -> dict[str, Any]:
        config = {}
        for f in fields(self):
            if f.name.startswith('_'):
                continue
            val = getattr(self, f.name)
            if val is not None:
                config[f.name] = val
        return {'name': self._codec_name, 'configuration': config}

    def compute_encoded_size(
        self, input_byte_length: int, chunk_spec: ArraySpec
    ) -> int:
        raise NotImplementedError


@dataclass(frozen=True)
class Packbits(_BytesBytesBase):
    """PackBits (RLE) codec for zarr v3.

    Standard TIFF/Apple PackBits algorithm, also used for
    DICOM RLE transfer syntax.
    """

    _codec_name: ClassVar[str] = 'imagecodecs_packbits'

    async def _decode_single(
        self, chunk_data: Buffer, chunk_spec: ArraySpec
    ) -> Buffer:
        chunk_bytes = chunk_data.to_bytes()

        def _decode() -> bytes:
            return bytes(packbits_decode(chunk_bytes))

        out = await asyncio.to_thread(_decode)
        return chunk_spec.prototype.buffer.from_bytes(out)

    async def _encode_single(
        self, chunk_data: Buffer, chunk_spec: ArraySpec
    ) -> Buffer:
        chunk_bytes = chunk_data.to_bytes()

        def _encode() -> bytes:
            return bytes(packbits_encode(chunk_bytes))

        out = await asyncio.to_thread(_encode)
        return chunk_spec.prototype.buffer.from_bytes(out)
