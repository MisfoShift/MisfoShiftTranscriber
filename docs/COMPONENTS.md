# Managed Components

This document records the update and release policy for components managed by MisfoShiftTranscriber.

## Source of truth

`src-tauri/components.json` is the machine-readable source of truth for:

- component identifier and type;
- recommended version;
- HTTPS download URL;
- SHA-256 digest;
- archive format and expected entrypoint;
- approximate download and required disk space;
- application-data installation directory;
- upstream project and license identifier.

Do not add download URLs or digests directly to Rust or TypeScript code.

## Update checklist

For every component update:

1. Download the exact artifact from the upstream project over HTTPS.
2. Verify that the release/tag and publisher are the intended upstream.
3. Calculate SHA-256 independently and record the exact byte size.
4. Inspect the archive and confirm its entrypoint and runtime dependencies.
5. Review the upstream license and build configuration.
6. Update `components.json`, `THIRD_PARTY_NOTICES.md`, and relevant README text together.
7. Test a clean install, interrupted download, digest mismatch, re-download, and offline transcription.
8. Keep whisper.cpp, model files, and FFmpeg out of the default application bundle. If a future release bundles FFmpeg, archive its license text, corresponding source, and matching BtbN build definitions as required by the LGPL.
9. For BtbN, pin the last successful monthly build covered by the upstream two-year retention policy. Do not pin an ordinary daily build, which is retained only temporarily. See the [BtbN release retention policy](https://github.com/BtbN/FFmpeg-Builds#release-retention-policy).

## Network policy

The component manager must not contact any server during normal startup or transcription. Network access occurs only after a user selects download, re-download, or recommended setup.

Downloads are written to a temporary directory under the application-data component root. An artifact is installed only after its digest is verified. The HTTP body is limited to the manifest download size plus 5%, with a minimum 8 MiB tolerance, and never beyond `required_space_bytes`. A larger `Content-Length` is rejected before writing; a streaming response that crosses the limit is stopped and its temporary file is removed. ZIP paths are constrained to the temporary extraction root, symbolic links are rejected, and extraction size is bounded by the manifest. Existing installations remain available until a verified replacement is ready.

## Current component set

- whisper.cpp `1.9.0`, Windows x64 CPU release, MIT.
- Whisper `small`, `medium`, and `large-v3` multilingual ggml models, MIT-designated upstream repository.
- BtbN FFmpeg `n8.1.2-50-g1a748fe2cd` Windows x64 LGPL static monthly build, LGPL-3.0-or-later.

See `THIRD_PARTY_NOTICES.md` for the exact artifacts and redistribution notes.
