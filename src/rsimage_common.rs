#[repr(i32)]
#[derive(Debug)]
pub enum RSIDecodeResult {
    Ok = 0,
    InvalidInput = -1,
    HeaderParseFailure = -2,
    FrameDecodeFailure = -3,
    AllocationFailure = -4,
}

#[repr(i32)]
#[derive(Debug, PartialEq)]
pub enum RSIPixelFormat {
    RGBA = 0,
    BGRA = 1,
}

#[repr(C)]
pub struct RSIDecodedImage {
    pub width: u32,
    pub height: u32,
    pub size: usize,
    pub image_data: *mut u8,
}

/// rsimage メモリアロケーション関数
///
/// # 引数
/// - `size`: 確保するメモリサイズ
///
/// # 戻り値
/// - 確保されたメモリのポインタ
#[unsafe(no_mangle)]
pub extern "system" fn rsimage_alloc(size: usize) -> *mut u8 {
    if size == 0 {
        return std::ptr::null_mut();
    }
    Box::leak(vec![0u8; size].into_boxed_slice()).as_mut_ptr()
}

/// rsimage メモリ解放関数
///
/// # 引数
/// - rsimageにより割り当てられたポインタ (`RsDecodedImage` の `image_data` など)
#[unsafe(no_mangle)]
pub extern "system" fn rsimage_free(image_data: *mut u8) {
    if image_data.is_null() {
        return;
    }

    unsafe {
        let _ = Vec::from_raw_parts(image_data, 0, 0); // メモリ解放
    }
}
