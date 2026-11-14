# Voice

Files the worker's `voice` job reads (P11). They ship as separate files beside the worker, unmodified.

- `silero-vad-v6.2-F32.gguf`: the voice detector that marks speech, so a dictation is cut at pauses. Silero VAD 6.2.3 (https://github.com/snakers4/silero-vad, commit `5cd7945`), converted to GGUF by handy-computer for transcribe.cpp: https://huggingface.co/handy-computer/silero-vad-v6.2-gguf at revision `a9b2d61c20ae4ddf3644edb34969e8185a485448`. 1,241,056 bytes, SHA-256 `0d871a800c420fdd5d1b17a3aea4296d1c39608f0ff4a9295eb76457eb1c88e6`. MIT; `LICENSE` and `NOTICE` come from the same revision.
