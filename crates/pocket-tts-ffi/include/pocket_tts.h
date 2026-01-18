#ifndef POCKET_TTS_H
#define POCKET_TTS_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct pocket_tts_model_t pocket_tts_model_t;
typedef struct pocket_tts_voice_state_t pocket_tts_voice_state_t;
typedef struct pocket_tts_stream_t pocket_tts_stream_t;

const char *pocket_tts_last_error_message(void);
void pocket_tts_clear_error(void);

pocket_tts_model_t *pocket_tts_model_load(const char *variant);
pocket_tts_model_t *pocket_tts_model_load_with_params(
    const char *variant,
    float temp,
    size_t lsd_decode_steps,
    float eos_threshold);
pocket_tts_model_t *pocket_tts_model_load_from_dir(
    const char *variant,
    const char *model_dir);
pocket_tts_model_t *pocket_tts_model_load_with_params_from_dir(
    const char *variant,
    const char *model_dir,
    float temp,
    size_t lsd_decode_steps,
    float eos_threshold);
void pocket_tts_model_free(pocket_tts_model_t *model);
uint32_t pocket_tts_model_sample_rate(const pocket_tts_model_t *model);

pocket_tts_voice_state_t *pocket_tts_voice_state_default(void);
pocket_tts_voice_state_t *pocket_tts_voice_state_from_path(
    const pocket_tts_model_t *model,
    const char *path);
pocket_tts_voice_state_t *pocket_tts_voice_state_from_audio_bytes(
    const pocket_tts_model_t *model,
    const uint8_t *bytes,
    size_t len);
pocket_tts_voice_state_t *pocket_tts_voice_state_from_prompt_bytes(
    const pocket_tts_model_t *model,
    const uint8_t *bytes,
    size_t len);
void pocket_tts_voice_state_free(pocket_tts_voice_state_t *state);

int pocket_tts_generate(
    const pocket_tts_model_t *model,
    const char *text,
    const pocket_tts_voice_state_t *voice_state,
    float **out_ptr,
    size_t *out_len);
int pocket_tts_generate_with_pauses(
    const pocket_tts_model_t *model,
    const char *text,
    const pocket_tts_voice_state_t *voice_state,
    float **out_ptr,
    size_t *out_len);

pocket_tts_stream_t *pocket_tts_stream_new(
    const pocket_tts_model_t *model,
    const char *text,
    const pocket_tts_voice_state_t *voice_state,
    int long_text);
int pocket_tts_stream_next(pocket_tts_stream_t *stream, float **out_ptr, size_t *out_len);
void pocket_tts_stream_free(pocket_tts_stream_t *stream);

void pocket_tts_audio_free(float *ptr, size_t len);

#ifdef __cplusplus
}
#endif

#endif
