use png::{Decoder, Transformations};
use crate::rsimage_common::*;

/// PNG データをデコードしてコールバック関数を呼び出すエクスポート関数.
///
/// # 引数
/// - `png_data`: PNG データが格納されたポインタ
/// - `png_data_size`: PNG データのサイズ（バイト数）
/// - `callback`: デコード完了時に呼び出されるコールバック関数のポインタ  
///
/// # 戻り値
/// - `RSIDecodeResult::Ok` (0): 正常終了
/// - その他の負の値: 各種エラーを示す
#[unsafe(no_mangle)]
pub extern "stdcall" fn rsimage_png_decode_memory(
    png_data: *const u8,
    png_data_size: u32,
    format: RSIPixelFormat,
    allocator: Option<extern "stdcall" fn(usize) -> *mut u8>,
    output: *mut RSIDecodedImage
) -> RSIDecodeResult {
    // 入力の妥当性チェック
    if png_data.is_null() || png_data_size == 0 {
        return RSIDecodeResult::InvalidInput;
    }

    // raw ポインタからスライスを作成
    let data_slice = unsafe { std::slice::from_raw_parts(png_data, png_data_size as usize) };

    // PNG デコーダの初期化
    let mut decoder = Decoder::new(data_slice);

    // 常にRGBAとして読み込む
    decoder.set_transformations(Transformations::EXPAND | Transformations::STRIP_16 | Transformations::ALPHA);

    let mut reader = match decoder.read_info() {
        Ok(reader) => reader,
        Err(_) => return RSIDecodeResult::HeaderParseFailure,
    };
    
    let buf_size = reader.output_buffer_size();
    let mut buf = Vec::with_capacity(buf_size);
    unsafe { buf.set_len(buf_size); }

    let info = match reader.next_frame(&mut buf) {
        Ok(info) => info,
        Err(_) => return RSIDecodeResult::FrameDecodeFailure,
    };

    let allocated_ptr = if let Some(alloc) = allocator {
        alloc(buf.len())
    } else {
        rsimage_alloc(buf.len()) // `allocator` が null の場合 `rsimage_alloc` を使用
    };

    if allocated_ptr.is_null() {
        return RSIDecodeResult::FrameDecodeFailure; // メモリアロケーション失敗
    }

    // u8->u32変換
    let buf32 = unsafe {
        std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u32, buf_size / 4)
    };
    
    if format==RSIPixelFormat::BGRA {
        // BGRA変換 (RとBを入れ替え)
        for pixel in buf32.iter_mut() {
            *pixel = (*pixel & 0xFF00FF00) | ((*pixel & 0x00FF0000) >> 16) | ((*pixel & 0x000000FF) << 16);
        }
    }

    unsafe {
        std::ptr::copy_nonoverlapping(buf.as_ptr(), allocated_ptr, buf_size);
        (*output).width = info.width;
        (*output).height = info.height;
        (*output).size = buf_size;
        (*output).image_data = allocated_ptr;
    }
    
    RSIDecodeResult::Ok
}
