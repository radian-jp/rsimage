use crate::rsimage_common::*;
use image::imageops::FilterType;
use image::load_from_memory;
use std::ffi::c_void;

impl RSIResizeFilter {
    fn to_image_filter(self) -> FilterType {
        match self {
            RSIResizeFilter::Nearest => FilterType::Nearest,
            RSIResizeFilter::Triangle => FilterType::Triangle,
            RSIResizeFilter::CatmullRom => FilterType::CatmullRom,
            RSIResizeFilter::Gaussian => FilterType::Gaussian,
            RSIResizeFilter::Lanczos3 => FilterType::Lanczos3,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn rsimage_generic_decode_memory(
    image_data: *const u8,
    image_data_size: u32,
    format: RSIPixelFormat,
    allocator: Option<extern "system" fn(usize) -> *mut u8>,
    output: *mut RSIDecodedImage,
) -> RSIDecodeResult {
    if image_data.is_null() || image_data_size == 0 || output.is_null() {
        return RSIDecodeResult::InvalidInput;
    }

    let data_slice = unsafe {
        std::slice::from_raw_parts(image_data, image_data_size as usize)
    };

    let dyn_img = match load_from_memory(data_slice) {
        Ok(img) => img,
        Err(_) => return RSIDecodeResult::HeaderParseFailure,
    };

    let rgba = dyn_img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    let mut buf = rgba.into_raw();
    let size = buf.len();

    if format == RSIPixelFormat::BGRA {
        for px in buf.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
    }

    let allocated_ptr = if let Some(alloc) = allocator {
        alloc(size)
    } else {
        rsimage_alloc(size)
    };

    if allocated_ptr.is_null() {
        return RSIDecodeResult::AllocationFailure;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(buf.as_ptr(), allocated_ptr, size);
        (*output).width = width;
        (*output).height = height;
        (*output).size = size;
        (*output).image_data = allocated_ptr;
    }

    RSIDecodeResult::Ok
}

#[unsafe(no_mangle)]
pub extern "system" fn rsimage_generic_decode_resize_memory(
    image_data: *const u8,
    image_data_size: u32,
    image_width: u32,
    image_height: u32,
    filter: RSIResizeFilter,
    format: RSIPixelFormat,
    allocator: Option<extern "system" fn(usize) -> *mut u8>,
    output: *mut RSIDecodedImage,
) -> RSIDecodeResult {
    if image_data.is_null() || image_data_size == 0 || output.is_null() {
        return RSIDecodeResult::InvalidInput;
    }

    if image_width == 0 || image_height == 0 {
        return RSIDecodeResult::InvalidInput;
    }

    let data_slice = unsafe {
        std::slice::from_raw_parts(image_data, image_data_size as usize)
    };

    let dyn_img = match load_from_memory(data_slice) {
        Ok(img) => img,
        Err(_) => return RSIDecodeResult::HeaderParseFailure,
    };

    let src_width = dyn_img.width();
    let src_height = dyn_img.height();

    // 縦横比維持で image_width × image_height 内に収めるリサイズ
    let (target_width, target_height) = if src_width <= image_width && src_height <= image_height {
        // 既に範囲内ならリサイズ不要
        (src_width, src_height)
    } else {
        let scale_w = image_width as f32 / src_width as f32;
        let scale_h = image_height as f32 / src_height as f32;
        let scale = scale_w.min(scale_h); // 小さい方を使う（範囲内に収まる）

        let w = (src_width as f32 * scale).round() as u32;
        let h = (src_height as f32 * scale).round() as u32;

        // 最小 1 ピクセル
        let w = if w == 0 { 1 } else { w };
        let h = if h == 0 { 1 } else { h };

        (w, h)
    };

    let resized = dyn_img.resize(target_width, target_height, filter.to_image_filter());

    let rgba = resized.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    let mut buf = rgba.into_raw();
    let size = buf.len();

    if format == RSIPixelFormat::BGRA {
        for px in buf.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
    }

    let allocated_ptr = if let Some(alloc) = allocator {
        alloc(size)
    } else {
        rsimage_alloc(size)
    };

    if allocated_ptr.is_null() {
        return RSIDecodeResult::AllocationFailure;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(buf.as_ptr(), allocated_ptr, size);
        (*output).width = width;
        (*output).height = height;
        (*output).size = size;
        (*output).image_data = allocated_ptr;
    }

    RSIDecodeResult::Ok
}

#[unsafe(no_mangle)]
pub extern "system" fn rsimage_generic_decode_callback(
    image_data: *const u8,
    image_data_size: u32,
    format: RSIPixelFormat,
    allocate_callback: RSIAllocateExternalBufferFn,
    user_data: *mut c_void,
    output: *mut RSIDecodedImage,
) -> RSIDecodeResult {
    if image_data.is_null() || image_data_size == 0 || output.is_null() {
        return RSIDecodeResult::InvalidInput;
    }

    let data_slice = unsafe {
        std::slice::from_raw_parts(image_data, image_data_size as usize)
    };

    let dyn_img = match load_from_memory(data_slice) {
        Ok(img) => img,
        Err(_) => return RSIDecodeResult::HeaderParseFailure,
    };

    let rgba = dyn_img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    let mut buf = rgba.into_raw();
    let size = buf.len();

    if format == RSIPixelFormat::BGRA {
        for px in buf.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
    }

    // コールバックを呼んで外部バッファを確保してもらう
    let mut external_buffer = RSIExternalBuffer {
        data: std::ptr::null_mut(),
        capacity: 0,
        image_object_handle: std::ptr::null_mut(),
    };

    let allocate_result = unsafe {
        allocate_callback(
            width,
            height,
            size,
            user_data,
            &mut external_buffer,
        )
    };

    if allocate_result!=RSIDecodeResult::Ok || external_buffer.data.is_null() || external_buffer.capacity < size {
        return RSIDecodeResult::AllocationFailure;
    }

    // 外部バッファに直接書き込み
    unsafe {
        std::ptr::copy_nonoverlapping(buf.as_ptr(), external_buffer.data, size);

        (*output).width = width;
        (*output).height = height;
        (*output).size = size;
        // Rust が確保したメモリではないので null
        (*output).image_data = std::ptr::null_mut();
        // C# から渡された image_object_handle をそのまま返す
        (*output).image_object_handle = external_buffer.image_object_handle;
    }

    RSIDecodeResult::Ok
}

#[unsafe(no_mangle)]
pub extern "system" fn rsimage_generic_decode_resize_callback(
    image_data: *const u8,
    image_data_size: u32,
    image_width: u32,
    image_height: u32,
    filter: RSIResizeFilter,
    format: RSIPixelFormat,
    allocate_callback: RSIAllocateExternalBufferFn,
    user_data: *mut c_void,
    output: *mut RSIDecodedImage,
) -> RSIDecodeResult {
    if image_data.is_null() || image_data_size == 0 || output.is_null() {
        return RSIDecodeResult::InvalidInput;
    }

    if image_width == 0 || image_height == 0 {
        return RSIDecodeResult::InvalidInput;
    }

    let data_slice = unsafe {
        std::slice::from_raw_parts(image_data, image_data_size as usize)
    };

    let dyn_img = match load_from_memory(data_slice) {
        Ok(img) => img,
        Err(_) => return RSIDecodeResult::HeaderParseFailure,
    };

    let src_width = dyn_img.width();
    let src_height = dyn_img.height();

    // 縦横比維持で image_width × image_height 内に収めるリサイズ
    let (target_width, target_height) = if src_width <= image_width && src_height <= image_height {
        // 既に範囲内ならリサイズ不要
        (src_width, src_height)
    } else {
        let scale_w = image_width as f32 / src_width as f32;
        let scale_h = image_height as f32 / src_height as f32;
        let scale = scale_w.min(scale_h); // 小さい方を使う（範囲内に収まる）

        let w = (src_width as f32 * scale).round() as u32;
        let h = (src_height as f32 * scale).round() as u32;

        // 最小 1 ピクセル
        let w = if w == 0 { 1 } else { w };
        let h = if h == 0 { 1 } else { h };

        (w, h)
    };

    let resized = dyn_img.resize(target_width, target_height, filter.to_image_filter());

    let rgba = resized.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    let mut buf = rgba.into_raw();
    let size = buf.len();

    if format == RSIPixelFormat::BGRA {
        for px in buf.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
    }

    // コールバックを呼んで外部バッファを確保してもらう
    let mut external_buffer = RSIExternalBuffer {
        data: std::ptr::null_mut(),
        capacity: 0,
        image_object_handle: std::ptr::null_mut(),
    };

    let allocate_result = unsafe {
        allocate_callback(
            width,
            height,
            size,
            user_data,
            &mut external_buffer,
        )
    };

    if allocate_result!=RSIDecodeResult::Ok || external_buffer.data.is_null() || external_buffer.capacity < size {
        return RSIDecodeResult::AllocationFailure;
    }

    // 外部バッファに直接書き込み
    unsafe {
        std::ptr::copy_nonoverlapping(buf.as_ptr(), external_buffer.data, size);

        (*output).width = width;
        (*output).height = height;
        (*output).size = size;
        (*output).image_data = std::ptr::null_mut();
        (*output).image_object_handle = external_buffer.image_object_handle;
    }

    RSIDecodeResult::Ok
}