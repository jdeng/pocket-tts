package pockettts

/*
#cgo CFLAGS: -I${SRCDIR}/../../crates/pocket-tts-ffi/include
#cgo darwin LDFLAGS: -L${SRCDIR}/../../target/release -lpocket_tts_ffi -Wl,-rpath,${SRCDIR}/../../target/release
#cgo linux LDFLAGS: -L${SRCDIR}/../../target/release -lpocket_tts_ffi -Wl,-rpath,${SRCDIR}/../../target/release
#include <stdlib.h>
#include "pocket_tts.h"
*/
import "C"

import (
	"errors"
	"unsafe"
)

type Model struct {
	ptr *C.pocket_tts_model_t
}

type VoiceState struct {
	ptr *C.pocket_tts_voice_state_t
}

type Stream struct {
	ptr *C.pocket_tts_stream_t
}

func lastError() error {
	msg := C.pocket_tts_last_error_message()
	if msg == nil {
		return errors.New("pocket-tts: unknown error")
	}
	err := C.GoString(msg)
	C.pocket_tts_clear_error()
	if err == "" {
		return errors.New("pocket-tts: unknown error")
	}
	return errors.New(err)
}

func Load(variant string) (*Model, error) {
	cVariant := C.CString(variant)
	defer C.free(unsafe.Pointer(cVariant))
	ptr := C.pocket_tts_model_load(cVariant)
	if ptr == nil {
		return nil, lastError()
	}
	return &Model{ptr: ptr}, nil
}

func LoadFromDir(variant string, modelDir string) (*Model, error) {
	cVariant := C.CString(variant)
	defer C.free(unsafe.Pointer(cVariant))
	cDir := C.CString(modelDir)
	defer C.free(unsafe.Pointer(cDir))
	ptr := C.pocket_tts_model_load_from_dir(cVariant, cDir)
	if ptr == nil {
		return nil, lastError()
	}
	return &Model{ptr: ptr}, nil
}

func LoadWithParams(variant string, temp float32, lsdDecodeSteps uint, eosThreshold float32) (*Model, error) {
	cVariant := C.CString(variant)
	defer C.free(unsafe.Pointer(cVariant))
	ptr := C.pocket_tts_model_load_with_params(
		cVariant,
		C.float(temp),
		C.size_t(lsdDecodeSteps),
		C.float(eosThreshold),
	)
	if ptr == nil {
		return nil, lastError()
	}
	return &Model{ptr: ptr}, nil
}

func LoadWithParamsFromDir(
	variant string,
	modelDir string,
	temp float32,
	lsdDecodeSteps uint,
	eosThreshold float32,
) (*Model, error) {
	cVariant := C.CString(variant)
	defer C.free(unsafe.Pointer(cVariant))
	cDir := C.CString(modelDir)
	defer C.free(unsafe.Pointer(cDir))
	ptr := C.pocket_tts_model_load_with_params_from_dir(
		cVariant,
		cDir,
		C.float(temp),
		C.size_t(lsdDecodeSteps),
		C.float(eosThreshold),
	)
	if ptr == nil {
		return nil, lastError()
	}
	return &Model{ptr: ptr}, nil
}

func (m *Model) Close() {
	if m == nil || m.ptr == nil {
		return
	}
	C.pocket_tts_model_free(m.ptr)
	m.ptr = nil
}

func (m *Model) SampleRate() uint32 {
	if m == nil || m.ptr == nil {
		return 0
	}
	return uint32(C.pocket_tts_model_sample_rate(m.ptr))
}

func NewDefaultVoiceState() (*VoiceState, error) {
	ptr := C.pocket_tts_voice_state_default()
	if ptr == nil {
		return nil, lastError()
	}
	return &VoiceState{ptr: ptr}, nil
}

func (m *Model) VoiceStateFromPath(path string) (*VoiceState, error) {
	if m == nil || m.ptr == nil {
		return nil, errors.New("pocket-tts: model is nil")
	}
	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))
	ptr := C.pocket_tts_voice_state_from_path(m.ptr, cPath)
	if ptr == nil {
		return nil, lastError()
	}
	return &VoiceState{ptr: ptr}, nil
}

func (m *Model) VoiceStateFromAudioBytes(data []byte) (*VoiceState, error) {
	if m == nil || m.ptr == nil {
		return nil, errors.New("pocket-tts: model is nil")
	}
	if len(data) == 0 {
		return nil, errors.New("pocket-tts: audio bytes empty")
	}
	ptr := C.pocket_tts_voice_state_from_audio_bytes(
		m.ptr,
		(*C.uint8_t)(unsafe.Pointer(&data[0])),
		C.size_t(len(data)),
	)
	if ptr == nil {
		return nil, lastError()
	}
	return &VoiceState{ptr: ptr}, nil
}

func (m *Model) VoiceStateFromPromptBytes(data []byte) (*VoiceState, error) {
	if m == nil || m.ptr == nil {
		return nil, errors.New("pocket-tts: model is nil")
	}
	if len(data) == 0 {
		return nil, errors.New("pocket-tts: prompt bytes empty")
	}
	ptr := C.pocket_tts_voice_state_from_prompt_bytes(
		m.ptr,
		(*C.uint8_t)(unsafe.Pointer(&data[0])),
		C.size_t(len(data)),
	)
	if ptr == nil {
		return nil, lastError()
	}
	return &VoiceState{ptr: ptr}, nil
}

func (s *VoiceState) Close() {
	if s == nil || s.ptr == nil {
		return
	}
	C.pocket_tts_voice_state_free(s.ptr)
	s.ptr = nil
}

func (m *Model) Generate(text string, voice *VoiceState) ([]float32, error) {
	if m == nil || m.ptr == nil {
		return nil, errors.New("pocket-tts: model is nil")
	}
	cText := C.CString(text)
	defer C.free(unsafe.Pointer(cText))
	var outPtr *C.float
	var outLen C.size_t
	var voicePtr *C.pocket_tts_voice_state_t
	if voice != nil {
		voicePtr = voice.ptr
	}
	status := C.pocket_tts_generate(m.ptr, cText, voicePtr, &outPtr, &outLen)
	if status != 0 {
		return nil, lastError()
	}
	return copyAndFree(outPtr, outLen), nil
}

func (m *Model) GenerateWithPauses(text string, voice *VoiceState) ([]float32, error) {
	if m == nil || m.ptr == nil {
		return nil, errors.New("pocket-tts: model is nil")
	}
	cText := C.CString(text)
	defer C.free(unsafe.Pointer(cText))
	var outPtr *C.float
	var outLen C.size_t
	var voicePtr *C.pocket_tts_voice_state_t
	if voice != nil {
		voicePtr = voice.ptr
	}
	status := C.pocket_tts_generate_with_pauses(m.ptr, cText, voicePtr, &outPtr, &outLen)
	if status != 0 {
		return nil, lastError()
	}
	return copyAndFree(outPtr, outLen), nil
}

func (m *Model) NewStream(text string, voice *VoiceState) (*Stream, error) {
	return m.newStream(text, voice, false)
}

func (m *Model) NewStreamLong(text string, voice *VoiceState) (*Stream, error) {
	return m.newStream(text, voice, true)
}

func (m *Model) newStream(text string, voice *VoiceState, longText bool) (*Stream, error) {
	if m == nil || m.ptr == nil {
		return nil, errors.New("pocket-tts: model is nil")
	}
	cText := C.CString(text)
	defer C.free(unsafe.Pointer(cText))
	var voicePtr *C.pocket_tts_voice_state_t
	if voice != nil {
		voicePtr = voice.ptr
	}
	var longFlag C.int
	if longText {
		longFlag = 1
	}
	ptr := C.pocket_tts_stream_new(m.ptr, cText, voicePtr, longFlag)
	if ptr == nil {
		return nil, lastError()
	}
	return &Stream{ptr: ptr}, nil
}

func (s *Stream) Next() ([]float32, bool, error) {
	if s == nil || s.ptr == nil {
		return nil, false, errors.New("pocket-tts: stream is nil")
	}
	var outPtr *C.float
	var outLen C.size_t
	status := C.pocket_tts_stream_next(s.ptr, &outPtr, &outLen)
	switch status {
	case 1:
		return copyAndFree(outPtr, outLen), true, nil
	case 0:
		return nil, false, nil
	default:
		return nil, false, lastError()
	}
}

func (s *Stream) Close() {
	if s == nil || s.ptr == nil {
		return
	}
	C.pocket_tts_stream_free(s.ptr)
	s.ptr = nil
}

func copyAndFree(ptr *C.float, len C.size_t) []float32 {
	if ptr == nil || len == 0 {
		return nil
	}
	count := int(len)
	src := unsafe.Slice((*float32)(unsafe.Pointer(ptr)), count)
	out := make([]float32, count)
	copy(out, src)
	C.pocket_tts_audio_free(ptr, len)
	return out
}
