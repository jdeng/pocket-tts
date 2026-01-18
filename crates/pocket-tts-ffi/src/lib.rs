use anyhow::Result;
use candle_core::Tensor;
use pocket_tts::{ModelState, TTSModel};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::ptr;
use std::sync::{Arc, Mutex, OnceLock};

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct pocket_tts_model_t {
    inner: Arc<TTSModel>,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct pocket_tts_voice_state_t {
    inner: ModelState,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct pocket_tts_stream_t {
    model: Arc<TTSModel>,
    voice_state: Box<ModelState>,
    iter: Box<dyn Iterator<Item = Result<Tensor>> + 'static>,
}

fn last_error() -> &'static Mutex<Option<CString>> {
    static LAST_ERROR: OnceLock<Mutex<Option<CString>>> = OnceLock::new();
    LAST_ERROR.get_or_init(|| Mutex::new(None))
}

fn set_last_error<E: std::fmt::Display>(err: E) {
    let msg = err.to_string();
    let c_msg = CString::new(msg).unwrap_or_else(|_| CString::new("error").unwrap());
    if let Ok(mut slot) = last_error().lock() {
        *slot = Some(c_msg);
    }
}

fn clear_last_error() {
    if let Ok(mut slot) = last_error().lock() {
        *slot = None;
    }
}

fn cstr_to_string(ptr: *const c_char) -> Result<String> {
    if ptr.is_null() {
        anyhow::bail!("null pointer");
    }
    let cstr = unsafe { CStr::from_ptr(ptr) };
    Ok(cstr.to_string_lossy().into_owned())
}

fn bytes_from_ptr<'a>(ptr: *const u8, len: usize) -> Result<&'a [u8]> {
    if len == 0 {
        anyhow::bail!("empty buffer");
    }
    if ptr.is_null() {
        anyhow::bail!("null pointer");
    }
    Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
}

fn tensor_to_vec(tensor: Tensor) -> Result<Vec<f32>> {
    let flat = tensor.flatten_all()?;
    let vec = flat.to_vec1::<f32>()?;
    Ok(vec)
}

fn alloc_audio_buffer(data: Vec<f32>, out_ptr: *mut *mut f32, out_len: *mut usize) {
    let mut boxed = data.into_boxed_slice();
    let ptr = boxed.as_mut_ptr();
    let len = boxed.len();
    std::mem::forget(boxed);
    unsafe {
        *out_ptr = ptr;
        *out_len = len;
    }
}

fn voice_state_from_path(model: &TTSModel, path: &str) -> Result<ModelState> {
    let path_obj = Path::new(path);
    let ext = path_obj
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "safetensors" {
        model.get_voice_state_from_prompt_file(path)
    } else {
        model.get_voice_state(path)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pocket_tts_last_error_message() -> *const c_char {
    if let Ok(slot) = last_error().lock() {
        if let Some(msg) = slot.as_ref() {
            return msg.as_ptr();
        }
    }
    ptr::null()
}

#[unsafe(no_mangle)]
pub extern "C" fn pocket_tts_clear_error() {
    clear_last_error();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pocket_tts_audio_free(ptr: *mut f32, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    unsafe {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pocket_tts_model_load(variant: *const c_char) -> *mut pocket_tts_model_t {
    clear_last_error();
    let variant = match cstr_to_string(variant) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return ptr::null_mut();
        }
    };

    match TTSModel::load(&variant) {
        Ok(model) => Box::into_raw(Box::new(pocket_tts_model_t {
            inner: Arc::new(model),
        })),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pocket_tts_model_load_with_params(
    variant: *const c_char,
    temp: f32,
    lsd_decode_steps: usize,
    eos_threshold: f32,
) -> *mut pocket_tts_model_t {
    clear_last_error();
    let variant = match cstr_to_string(variant) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return ptr::null_mut();
        }
    };

    match TTSModel::load_with_params(&variant, temp, lsd_decode_steps, eos_threshold) {
        Ok(model) => Box::into_raw(Box::new(pocket_tts_model_t {
            inner: Arc::new(model),
        })),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pocket_tts_model_load_from_dir(
    variant: *const c_char,
    model_dir: *const c_char,
) -> *mut pocket_tts_model_t {
    clear_last_error();
    let variant = match cstr_to_string(variant) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return ptr::null_mut();
        }
    };
    let model_dir = match cstr_to_string(model_dir) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return ptr::null_mut();
        }
    };

    match TTSModel::load_from_dir(&variant, model_dir) {
        Ok(model) => Box::into_raw(Box::new(pocket_tts_model_t {
            inner: Arc::new(model),
        })),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pocket_tts_model_load_with_params_from_dir(
    variant: *const c_char,
    model_dir: *const c_char,
    temp: f32,
    lsd_decode_steps: usize,
    eos_threshold: f32,
) -> *mut pocket_tts_model_t {
    clear_last_error();
    let variant = match cstr_to_string(variant) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return ptr::null_mut();
        }
    };
    let model_dir = match cstr_to_string(model_dir) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return ptr::null_mut();
        }
    };

    match TTSModel::load_with_params_from_dir(&variant, model_dir, temp, lsd_decode_steps, eos_threshold) {
        Ok(model) => Box::into_raw(Box::new(pocket_tts_model_t {
            inner: Arc::new(model),
        })),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pocket_tts_model_free(model: *mut pocket_tts_model_t) {
    if model.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(model));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pocket_tts_model_sample_rate(model: *const pocket_tts_model_t) -> u32 {
    if model.is_null() {
        return 0;
    }
    let model = unsafe { &*model };
    model.inner.sample_rate as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn pocket_tts_voice_state_default() -> *mut pocket_tts_voice_state_t {
    clear_last_error();
    let state = pocket_tts::voice_state::init_states(1, 0);
    Box::into_raw(Box::new(pocket_tts_voice_state_t { inner: state }))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pocket_tts_voice_state_from_path(
    model: *const pocket_tts_model_t,
    path: *const c_char,
) -> *mut pocket_tts_voice_state_t {
    clear_last_error();
    if model.is_null() {
        set_last_error("model is null");
        return ptr::null_mut();
    }
    let path = match cstr_to_string(path) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return ptr::null_mut();
        }
    };
    let model = unsafe { &*model };
    match voice_state_from_path(&model.inner, &path) {
        Ok(state) => Box::into_raw(Box::new(pocket_tts_voice_state_t { inner: state })),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pocket_tts_voice_state_from_audio_bytes(
    model: *const pocket_tts_model_t,
    bytes: *const u8,
    len: usize,
) -> *mut pocket_tts_voice_state_t {
    clear_last_error();
    if model.is_null() {
        set_last_error("model is null");
        return ptr::null_mut();
    }
    let bytes = match bytes_from_ptr(bytes, len) {
        Ok(b) => b,
        Err(e) => {
            set_last_error(e);
            return ptr::null_mut();
        }
    };
    let model = unsafe { &*model };
    match model.inner.get_voice_state_from_bytes(bytes) {
        Ok(state) => Box::into_raw(Box::new(pocket_tts_voice_state_t { inner: state })),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pocket_tts_voice_state_from_prompt_bytes(
    model: *const pocket_tts_model_t,
    bytes: *const u8,
    len: usize,
) -> *mut pocket_tts_voice_state_t {
    clear_last_error();
    if model.is_null() {
        set_last_error("model is null");
        return ptr::null_mut();
    }
    let bytes = match bytes_from_ptr(bytes, len) {
        Ok(b) => b,
        Err(e) => {
            set_last_error(e);
            return ptr::null_mut();
        }
    };
    let model = unsafe { &*model };
    match model.inner.get_voice_state_from_prompt_bytes(bytes) {
        Ok(state) => Box::into_raw(Box::new(pocket_tts_voice_state_t { inner: state })),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pocket_tts_voice_state_free(state: *mut pocket_tts_voice_state_t) {
    if state.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(state));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pocket_tts_generate(
    model: *const pocket_tts_model_t,
    text: *const c_char,
    voice_state: *const pocket_tts_voice_state_t,
    out_ptr: *mut *mut f32,
    out_len: *mut usize,
) -> c_int {
    clear_last_error();
    if model.is_null() {
        set_last_error("model is null");
        return -1;
    }
    if out_ptr.is_null() || out_len.is_null() {
        set_last_error("output pointers are null");
        return -1;
    }
    let text = match cstr_to_string(text) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return -1;
        }
    };
    let model = unsafe { &*model };
    let default_state = if voice_state.is_null() {
        Some(pocket_tts::voice_state::init_states(1, 0))
    } else {
        None
    };
    let state_ref = match default_state.as_ref() {
        Some(state) => state,
        None => unsafe { &(*voice_state).inner },
    };

    match model.inner.generate(&text, state_ref) {
        Ok(tensor) => match tensor_to_vec(tensor) {
            Ok(samples) => {
                alloc_audio_buffer(samples, out_ptr, out_len);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        },
        Err(e) => {
            set_last_error(e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pocket_tts_generate_with_pauses(
    model: *const pocket_tts_model_t,
    text: *const c_char,
    voice_state: *const pocket_tts_voice_state_t,
    out_ptr: *mut *mut f32,
    out_len: *mut usize,
) -> c_int {
    clear_last_error();
    if model.is_null() {
        set_last_error("model is null");
        return -1;
    }
    if out_ptr.is_null() || out_len.is_null() {
        set_last_error("output pointers are null");
        return -1;
    }
    let text = match cstr_to_string(text) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return -1;
        }
    };
    let model = unsafe { &*model };
    let default_state = if voice_state.is_null() {
        Some(pocket_tts::voice_state::init_states(1, 0))
    } else {
        None
    };
    let state_ref = match default_state.as_ref() {
        Some(state) => state,
        None => unsafe { &(*voice_state).inner },
    };

    match model.inner.generate_with_pauses(&text, state_ref) {
        Ok(tensor) => match tensor_to_vec(tensor) {
            Ok(samples) => {
                alloc_audio_buffer(samples, out_ptr, out_len);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        },
        Err(e) => {
            set_last_error(e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pocket_tts_stream_new(
    model: *const pocket_tts_model_t,
    text: *const c_char,
    voice_state: *const pocket_tts_voice_state_t,
    long_text: c_int,
) -> *mut pocket_tts_stream_t {
    clear_last_error();
    if model.is_null() {
        set_last_error("model is null");
        return ptr::null_mut();
    }
    let text = match cstr_to_string(text) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return ptr::null_mut();
        }
    };

    let model = unsafe { &*model };
    let voice_state_owned = if voice_state.is_null() {
        pocket_tts::voice_state::init_states(1, 0)
    } else {
        unsafe { (*voice_state).inner.clone() }
    };

    let model_arc = model.inner.clone();
    let voice_state_box = Box::new(voice_state_owned);
    let long_text = long_text != 0;

    let iter = {
        let model_ref: &TTSModel = model_arc.as_ref();
        let voice_ref: &ModelState = voice_state_box.as_ref();
        let iter: Box<dyn Iterator<Item = Result<Tensor>> + '_> = if long_text {
            Box::new(model_ref.generate_stream_long(&text, voice_ref))
        } else {
            model_ref.generate_stream(&text, voice_ref)
        };
        // SAFETY: The iterator borrows model_ref/voice_ref, which are owned by
        // model_arc/voice_state_box stored in the stream handle for the stream's lifetime.
        unsafe {
            std::mem::transmute::<
                Box<dyn Iterator<Item = Result<Tensor>> + '_>,
                Box<dyn Iterator<Item = Result<Tensor>> + 'static>,
            >(iter)
        }
    };

    Box::into_raw(Box::new(pocket_tts_stream_t {
        model: model_arc,
        voice_state: voice_state_box,
        iter,
    }))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pocket_tts_stream_next(
    stream: *mut pocket_tts_stream_t,
    out_ptr: *mut *mut f32,
    out_len: *mut usize,
) -> c_int {
    clear_last_error();
    if stream.is_null() {
        set_last_error("stream is null");
        return -1;
    }
    if out_ptr.is_null() || out_len.is_null() {
        set_last_error("output pointers are null");
        return -1;
    }

    let stream = unsafe { &mut *stream };
    match stream.iter.next() {
        None => 0,
        Some(Ok(tensor)) => match tensor_to_vec(tensor) {
            Ok(samples) => {
                alloc_audio_buffer(samples, out_ptr, out_len);
                1
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        },
        Some(Err(e)) => {
            set_last_error(e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pocket_tts_stream_free(stream: *mut pocket_tts_stream_t) {
    if stream.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(stream));
    }
}
