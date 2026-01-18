package main

import (
	"fmt"
	"log"
	"os"
	"path/filepath"
	"runtime"

	"github.com/babybirdprd/pocket-tts/bindings/go"
)

func main() {
	if os.Getenv("HF_TOKEN") == "" {
		log.Println("HF_TOKEN is not set; model download may fail for gated weights")
	}

	variant := os.Getenv("POCKET_TTS_VARIANT")
	if variant == "" {
		variant = "b6369a24"
	}
	modelDir := os.Getenv("POCKET_TTS_MODEL_DIR")
	var model *pockettts.Model
	var err error
	if modelDir != "" {
		model, err = pockettts.LoadFromDir(variant, modelDir)
	} else {
		model, err = pockettts.Load(variant)
	}
	if err != nil {
		log.Fatal(err)
	}
	defer model.Close()

	voicePath := os.Getenv("POCKET_TTS_VOICE_PATH")
	if voicePath == "" {
		voicePath = defaultVoicePath()
	}
	if _, err := os.Stat(voicePath); err != nil {
		log.Fatalf("voice path not found: %s: %v", voicePath, err)
	}

	voice, err := model.VoiceStateFromPath(voicePath)
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

func defaultVoicePath() string {
	_, file, _, ok := runtime.Caller(0)
	if !ok {
		return "assets/ref.wav"
	}
	return filepath.Join(filepath.Dir(file), "..", "..", "..", "assets", "ref.wav")
}
