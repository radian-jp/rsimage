use turbojpeg::{Decompressor, PixelFormat};
use crate::rsimage_common::*;

/// JPEG データをデコードしてコールバック関数を呼び出すエクスポート関数.
///
/// # 引数
/// - `jpeg_data`: JPEG データのポインタ
/// - `jpeg_data_size`: JPEG データのサイズ（バイト数）
/// - `format`: 出力フォーマット (RGBA / BGRA)
/// - `allocator`: メモリアロケータ関数ポインタ（nullの場合はRust側で割り当て）
/// - `output`: デコード結果を格納するポインタ
///
/// # 戻り値
/// - `RSIDecodeResult::Ok` (0): 正常終了
/// - その他の負の値: 各種エラー
#[unsafe(no_mangle)]
pub extern "stdcall" fn rsimage_jpg_decode_memory(
    jpeg_data: *const u8,
    jpeg_data_size: u32,
    format: RSIPixelFormat,
    allocator: Option<extern "stdcall" fn(usize) -> *mut u8>,
    output: *mut RSIDecodedImage
) -> RSIDecodeResult {
    if jpeg_data.is_null() || jpeg_data_size == 0 || output.is_null() {
        return RSIDecodeResult::InvalidInput;
    }

    let data_slice = unsafe { std::slice::from_raw_parts(jpeg_data, jpeg_data_size as usize) };

    let mut decompressor = match Decompressor::new() {
        Ok(d) => d,
        Err(_) => return RSIDecodeResult::HeaderParseFailure,
    };

    let header = match decompressor.read_header(data_slice) {
        Ok(h) => h,
        Err(_) => return RSIDecodeResult::HeaderParseFailure,
    };

    let pixel_format = match format {
        RSIPixelFormat::RGBA => PixelFormat::RGBA,
        RSIPixelFormat::BGRA => PixelFormat::BGRA,
    };

    let buf_size = (header.width * header.height * 4) as usize;
    let mut buf = vec![0u8; buf_size];

    let image = turbojpeg::Image {
        pixels: buf.as_mut_slice(),
        width: header.width as usize,
        height: header.height as usize,
        pitch: header.width as usize * pixel_format.size(),
        format: pixel_format,
    };

    if decompressor.decompress(data_slice, image).is_err() {
        return RSIDecodeResult::FrameDecodeFailure;
    }

    let allocated_ptr = if let Some(alloc) = allocator {
        alloc(buf.len())
    } else {
        rsimage_alloc(buf.len()) // `allocator` が null の場合 `rsimage_alloc` を使用
    };

    if allocated_ptr.is_null() {
        return RSIDecodeResult::AllocationFailure;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(buf.as_ptr(), allocated_ptr, buf_size);
        (*output).width = header.width as u32;
        (*output).height = header.height as u32;
        (*output).size = buf_size;
        (*output).image_data = allocated_ptr;
    }

    RSIDecodeResult::Ok
}
