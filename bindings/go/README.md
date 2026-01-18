# Go bindings for pocket-tts

This package wraps the `pocket-tts` Rust library via a small C ABI layer.

## Build the Rust FFI library

```bash
cargo build -p pocket-tts-ffi --release
```

The shared library will be in `target/release`:

- macOS: `libpocket_tts_ffi.dylib`
- Linux: `libpocket_tts_ffi.so`

## Use from Go

From `bindings/go`:

```bash
go test ./...
```

If the loader cannot find the shared library, set the library path:

```bash
export DYLD_LIBRARY_PATH="$PWD/../../target/release"   # macOS
export LD_LIBRARY_PATH="$PWD/../../target/release"     # Linux
```

## Local model directory (HF clone)

If you want to run fully offline, you can clone/download the model files locally and load
from a directory. The directory must contain:

- `tts_<variant>.safetensors` (e.g. `tts_b6369a24.safetensors`)
- `tokenizer.model`

One way to populate a local directory with Hugging Face:

```bash
# Requires HF_TOKEN with access to kyutai/pocket-tts
hf download kyutai/pocket-tts tts_b6369a24.safetensors \
  --local-dir /path/to/pocket-tts-model
hf download kyutai/pocket-tts-without-voice-cloning tokenizer.model \
  --local-dir /path/to/pocket-tts-model
```

Alternatively, using git LFS:

```bash
git lfs install
git clone https://huggingface.co/kyutai/pocket-tts /path/to/pocket-tts-model
git clone https://huggingface.co/kyutai/pocket-tts-without-voice-cloning /tmp/pocket-tts-tokenizer
cp /tmp/pocket-tts-tokenizer/tokenizer.model /path/to/pocket-tts-model/
```

Then load from Go:

```go
model, err := pockettts.LoadFromDir("b6369a24", "/path/to/pocket-tts-model")
```

## Use locally with go.mod replace

In your Go project's `go.mod`:

```text
require github.com/babybirdprd/pocket-tts/bindings/go v0.0.0

replace github.com/babybirdprd/pocket-tts/bindings/go => /path/to/pocket-tts/bindings/go
```

## Example

```go
package main

import (
	"fmt"
	"log"

	"github.com/babybirdprd/pocket-tts/bindings/go"
)

func main() {
	model, err := pockettts.Load("b6369a24")
	if err != nil {
		log.Fatal(err)
	}
	defer model.Close()

	voice, err := model.VoiceStateFromPath("assets/ref.wav")
	if err != nil {
		log.Fatal(err)
	}
	defer voice.Close()

	stream, err := model.NewStream("Hello from Go.", voice)
	if err != nil {
		log.Fatal(err)
	}
	defer stream.Close()

	var samples int
	for {
		chunk, ok, err := stream.Next()
		if err != nil {
			log.Fatal(err)
		}
		if !ok {
			break
		}
		samples += len(chunk)
	}
	fmt.Printf("generated %d samples at %d Hz\n", samples, model.SampleRate())
}
```

WAV encoding is intentionally left to Go libraries.
