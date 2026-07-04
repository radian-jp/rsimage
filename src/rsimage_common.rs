use std::ffi::c_void;

#[repr(i32)]
#[derive(Debug, PartialEq)]
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
    
    // Rust側で割り当てたポインタ(rsimage_free での解放が必要)
    pub image_data: *mut u8,

    /// 他言語から渡された画像オブジェクトへのハンドル
    pub image_object_handle: *mut std::ffi::c_void,
}

/// 外部バッファ情報（他言語から渡される）
#[repr(C)]
pub struct RSIExternalBuffer {
    /// ピクセルバッファ先頭
    pub data: *mut u8,

    /// バッファサイズ
    pub capacity: usize,

    /// 画像オブジェクト自体へのハンドル
    /// 例: GCHandle.ToIntPtr(handle) で得た IntPtr
    pub image_object_handle: *mut c_void,
}

/// 他言語から渡されるバッファ確保コールバック
/// width, height, required_bytes: 必要なサイズ
/// user_data: 他言語から渡される任意のデータ
/// out_buffer: 返すバッファ情報
/// 戻り値: 成功時は Ok, 割り当て失敗時は AllocationFailure
pub type RSIAllocateExternalBufferFn = unsafe extern "system" fn(
    width: u32,
    height: u32,
    required_bytes: usize,
    user_data: *mut c_void,
    out_buffer: *mut RSIExternalBuffer,
) -> RSIDecodeResult;

/// リサイズフィルタ選択
#[repr(i32)]
#[derive(Debug, PartialEq)]
pub enum RSIResizeFilter {
    Nearest = 0,      // 最近傍補間
    Triangle = 1,     // 双一次補間 (バイリニア)
    CatmullRom = 2,   // 双三次補間 (Catmull-Rom)
    Gaussian = 3,     // ガウス補間
    Lanczos3 = 4,     // ランツォス補間 (ウィンドウ 3)
}

#[cfg(not(target_os = "windows"))]
unsafe extern "C" {
    pub fn malloc(size: usize) -> *mut u8;
    pub fn free(ptr: *mut u8);
}

#[cfg(target_os = "windows")]
unsafe extern "system" {
    pub fn malloc(size: usize) -> *mut u8;
    pub fn free(ptr: *mut u8);
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
    unsafe { malloc(size) }
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
    unsafe { free(image_data) };
}
