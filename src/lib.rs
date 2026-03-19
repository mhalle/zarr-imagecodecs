use numpy::ndarray::{ArrayD, IxDyn};
use numpy::{IntoPyArray, PyArrayDyn, PyReadonlyArrayDyn};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

mod jpeg2k;
mod jpeg;
mod jpegxl;
mod png_codec;
mod webp;
mod avif_codec;
mod tiff_codec;
mod jpegls;
mod packbits;

#[pymodule]
fn _zarr_imagecodecs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(jpeg2k_encode, m)?)?;
    m.add_function(wrap_pyfunction!(jpeg2k_decode, m)?)?;
    m.add_function(wrap_pyfunction!(jpeg_encode, m)?)?;
    m.add_function(wrap_pyfunction!(jpeg_decode, m)?)?;
    m.add_function(wrap_pyfunction!(jpegxl_decode, m)?)?;
    m.add_function(wrap_pyfunction!(png_encode, m)?)?;
    m.add_function(wrap_pyfunction!(png_decode, m)?)?;
    m.add_function(wrap_pyfunction!(webp_encode, m)?)?;
    m.add_function(wrap_pyfunction!(webp_decode, m)?)?;
    m.add_function(wrap_pyfunction!(encode_avif, m)?)?;
    m.add_function(wrap_pyfunction!(decode_avif, m)?)?;
    m.add_function(wrap_pyfunction!(tiff_encode, m)?)?;
    m.add_function(wrap_pyfunction!(tiff_decode, m)?)?;
    m.add_function(wrap_pyfunction!(jpegls_encode, m)?)?;
    m.add_function(wrap_pyfunction!(jpegls_decode, m)?)?;
    m.add_function(wrap_pyfunction!(packbits_encode, m)?)?;
    m.add_function(wrap_pyfunction!(packbits_decode, m)?)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// JPEG 2000
// ---------------------------------------------------------------------------

#[pyfunction]
#[pyo3(signature = (data, *, level=None, reversible=false, num_resolutions=None))]
fn jpeg2k_encode<'py>(
    py: Python<'py>,
    data: PyReadonlyArrayDyn<'py, u8>,
    level: Option<f32>,
    reversible: bool,
    num_resolutions: Option<u32>,
) -> PyResult<Bound<'py, PyBytes>> {
    let array = data.as_array();
    let shape: Vec<usize> = array.shape().to_vec();
    let buf = array.as_slice().ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err("array must be contiguous")
    })?;
    let encoded = jpeg2k::encode(buf, &shape, level, reversible, num_resolutions)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, &encoded))
}

#[pyfunction]
#[pyo3(signature = (data, shape))]
fn jpeg2k_decode<'py>(
    py: Python<'py>,
    data: &Bound<'py, PyBytes>,
    shape: Vec<usize>,
) -> PyResult<Bound<'py, PyArrayDyn<u8>>> {
    let buf = data.as_bytes();
    let decoded = jpeg2k::decode(buf)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let arr = ArrayD::from_shape_vec(IxDyn(&shape), decoded)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    Ok(arr.into_pyarray(py))
}

// ---------------------------------------------------------------------------
// JPEG
// ---------------------------------------------------------------------------

#[pyfunction]
#[pyo3(signature = (data, *, quality=90))]
fn jpeg_encode<'py>(
    py: Python<'py>,
    data: PyReadonlyArrayDyn<'py, u8>,
    quality: u8,
) -> PyResult<Bound<'py, PyBytes>> {
    let array = data.as_array();
    let shape: Vec<usize> = array.shape().to_vec();
    let buf = array.as_slice().ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err("array must be contiguous")
    })?;
    let encoded = jpeg::encode(buf, &shape, quality)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, &encoded))
}

#[pyfunction]
#[pyo3(signature = (data, shape))]
fn jpeg_decode<'py>(
    py: Python<'py>,
    data: &Bound<'py, PyBytes>,
    shape: Vec<usize>,
) -> PyResult<Bound<'py, PyArrayDyn<u8>>> {
    let buf = data.as_bytes();
    let decoded = jpeg::decode(buf)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let arr = ArrayD::from_shape_vec(IxDyn(&shape), decoded)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    Ok(arr.into_pyarray(py))
}

// ---------------------------------------------------------------------------
// JPEG XL (decode only - pure Rust via jxl-oxide)
// ---------------------------------------------------------------------------

#[pyfunction]
#[pyo3(signature = (data, shape))]
fn jpegxl_decode<'py>(
    py: Python<'py>,
    data: &Bound<'py, PyBytes>,
    shape: Vec<usize>,
) -> PyResult<Bound<'py, PyArrayDyn<u8>>> {
    let buf = data.as_bytes();
    let decoded = jpegxl::decode(buf)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let arr = ArrayD::from_shape_vec(IxDyn(&shape), decoded)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    Ok(arr.into_pyarray(py))
}

// ---------------------------------------------------------------------------
// PNG
// ---------------------------------------------------------------------------

#[pyfunction]
#[pyo3(signature = (data, *, level=None))]
fn png_encode<'py>(
    py: Python<'py>,
    data: PyReadonlyArrayDyn<'py, u8>,
    level: Option<u8>,
) -> PyResult<Bound<'py, PyBytes>> {
    let array = data.as_array();
    let shape: Vec<usize> = array.shape().to_vec();
    let buf = array.as_slice().ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err("array must be contiguous")
    })?;
    let encoded = png_codec::encode(buf, &shape, level)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, &encoded))
}

#[pyfunction]
#[pyo3(signature = (data, shape))]
fn png_decode<'py>(
    py: Python<'py>,
    data: &Bound<'py, PyBytes>,
    shape: Vec<usize>,
) -> PyResult<Bound<'py, PyArrayDyn<u8>>> {
    let buf = data.as_bytes();
    let decoded = png_codec::decode(buf)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let arr = ArrayD::from_shape_vec(IxDyn(&shape), decoded)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    Ok(arr.into_pyarray(py))
}

// ---------------------------------------------------------------------------
// WebP
// ---------------------------------------------------------------------------

#[pyfunction]
#[pyo3(signature = (data, *, quality=None, lossless=false))]
fn webp_encode<'py>(
    py: Python<'py>,
    data: PyReadonlyArrayDyn<'py, u8>,
    quality: Option<f32>,
    lossless: bool,
) -> PyResult<Bound<'py, PyBytes>> {
    let array = data.as_array();
    let shape: Vec<usize> = array.shape().to_vec();
    let buf = array.as_slice().ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err("array must be contiguous")
    })?;
    let encoded = webp::encode(buf, &shape, quality, lossless)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, &encoded))
}

#[pyfunction]
#[pyo3(signature = (data, shape))]
fn webp_decode<'py>(
    py: Python<'py>,
    data: &Bound<'py, PyBytes>,
    shape: Vec<usize>,
) -> PyResult<Bound<'py, PyArrayDyn<u8>>> {
    let buf = data.as_bytes();
    let decoded = webp::decode(buf)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let arr = ArrayD::from_shape_vec(IxDyn(&shape), decoded)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    Ok(arr.into_pyarray(py))
}

// ---------------------------------------------------------------------------
// AVIF
// ---------------------------------------------------------------------------

#[pyfunction]
#[pyo3(name = "avif_encode", signature = (data, *, quality=None, speed=None))]
fn encode_avif<'py>(
    py: Python<'py>,
    data: PyReadonlyArrayDyn<'py, u8>,
    quality: Option<f32>,
    speed: Option<u8>,
) -> PyResult<Bound<'py, PyBytes>> {
    let array = data.as_array();
    let shape: Vec<usize> = array.shape().to_vec();
    let buf = array.as_slice().ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err("array must be contiguous")
    })?;
    let encoded = avif_codec::encode(buf, &shape, quality, speed)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, &encoded))
}

#[pyfunction]
#[pyo3(name = "avif_decode", signature = (data, shape))]
fn decode_avif<'py>(
    py: Python<'py>,
    data: &Bound<'py, PyBytes>,
    shape: Vec<usize>,
) -> PyResult<Bound<'py, PyArrayDyn<u8>>> {
    let buf = data.as_bytes();
    let decoded = avif_codec::decode(buf)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let arr = ArrayD::from_shape_vec(IxDyn(&shape), decoded)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    Ok(arr.into_pyarray(py))
}

// ---------------------------------------------------------------------------
// TIFF
// ---------------------------------------------------------------------------

#[pyfunction]
#[pyo3(signature = (data, *, compression=None))]
fn tiff_encode<'py>(
    py: Python<'py>,
    data: PyReadonlyArrayDyn<'py, u8>,
    compression: Option<String>,
) -> PyResult<Bound<'py, PyBytes>> {
    let array = data.as_array();
    let shape: Vec<usize> = array.shape().to_vec();
    let buf = array.as_slice().ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err("array must be contiguous")
    })?;
    let encoded = tiff_codec::encode(buf, &shape, compression.as_deref())
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, &encoded))
}

#[pyfunction]
#[pyo3(signature = (data, shape))]
fn tiff_decode<'py>(
    py: Python<'py>,
    data: &Bound<'py, PyBytes>,
    shape: Vec<usize>,
) -> PyResult<Bound<'py, PyArrayDyn<u8>>> {
    let buf = data.as_bytes();
    let decoded = tiff_codec::decode(buf)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let arr = ArrayD::from_shape_vec(IxDyn(&shape), decoded)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    Ok(arr.into_pyarray(py))
}

// ---------------------------------------------------------------------------
// JPEG-LS
// ---------------------------------------------------------------------------

#[pyfunction]
#[pyo3(signature = (data, *, near=0))]
fn jpegls_encode<'py>(
    py: Python<'py>,
    data: PyReadonlyArrayDyn<'py, u8>,
    near: i32,
) -> PyResult<Bound<'py, PyBytes>> {
    let array = data.as_array();
    let shape: Vec<usize> = array.shape().to_vec();
    let buf = array.as_slice().ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err("array must be contiguous")
    })?;
    let encoded = jpegls::encode(buf, &shape, near)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, &encoded))
}

#[pyfunction]
#[pyo3(signature = (data, shape))]
fn jpegls_decode<'py>(
    py: Python<'py>,
    data: &Bound<'py, PyBytes>,
    shape: Vec<usize>,
) -> PyResult<Bound<'py, PyArrayDyn<u8>>> {
    let buf = data.as_bytes();
    let decoded = jpegls::decode(buf)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let arr = ArrayD::from_shape_vec(IxDyn(&shape), decoded)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    Ok(arr.into_pyarray(py))
}

// ---------------------------------------------------------------------------
// PackBits (RLE)
// ---------------------------------------------------------------------------

#[pyfunction]
#[pyo3(signature = (data,))]
fn packbits_encode<'py>(
    py: Python<'py>,
    data: &Bound<'py, PyBytes>,
) -> PyResult<Bound<'py, PyBytes>> {
    let buf = data.as_bytes();
    let encoded = packbits::encode(buf);
    Ok(PyBytes::new(py, &encoded))
}

#[pyfunction]
#[pyo3(signature = (data,))]
fn packbits_decode<'py>(
    py: Python<'py>,
    data: &Bound<'py, PyBytes>,
) -> PyResult<Bound<'py, PyBytes>> {
    let buf = data.as_bytes();
    let decoded = packbits::decode(buf)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, &decoded))
}
