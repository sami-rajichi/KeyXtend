# Voice

Files the worker's `voice` job reads (P11). They ship as separate files beside the worker, unmodified.

- `silero-vad-v6.2-F32.gguf`: the voice detector that marks speech, so a dictation is cut at pauses. Silero VAD 6.2.3 (https://github.com/snakers4/silero-vad, commit `5cd7945`), converted to GGUF by handy-computer for transcribe.cpp: https://huggingface.co/handy-computer/silero-vad-v6.2-gguf at revision `a9b2d61c20ae4ddf3644edb34969e8185a485448`. 1,241,056 bytes, SHA-256 `0d871a800c420fdd5d1b17a3aea4296d1c39608f0ff4a9295eb76457eb1c88e6`. MIT; `LICENSE` and `NOTICE` come from the same revision.
- `check-en.wav`, `check-fr.wav`, `check-ar.wav`: the graphics card's self-test clips, one sentence each, read on the card and on the processor. From FLEURS (Conneau et al., 2022, Google), test split at revision `70bb2e84b976b7e960aa89f1c648e09c59f894dd`, unmodified: WAV, 32-bit float, mono, 16 kHz. CC-BY-4.0, see `LICENSE-CLIPS.txt`.

  | File | FLEURS clip | Bytes | SHA-256 | Sentence |
  |---|---|---|---|---|
  | `check-en.wav` | en_us 1677, `1731561740051730459.wav` | 380218 | `5a5a12d06c0a6acaf0a0d737d7c980bce91ec3f652cfa537b3cbaf8e10845dbc` | Today, the only insects that cannot fold back their wings are dragon flies and mayflies. |
  | `check-fr.wav` | fr_fr 1720, `15483129155942250394.wav` | 384058 | `04ffc415ce6da5ed5b190286de1b456aaee0b34e1e322d79187ca72ab27ca795` | Le chocolat chaud est conforme aux normes belges. Les jus de fruits sont chers mais excellents. |
  | `check-ar.wav` | ar_eg 1778 | 384058 | `28aacbf6c32ae30aaed5671b13d81be9ea7bbef9a32244a5e754703395414361` | يتصاعد من المصنع دخاناً أبيض، وهذا ما أظهرته تقارير تلفزيونية. |
