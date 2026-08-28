# Third-Party Notices

MisfoShiftTranscriber is published by MisfoShift and can download or use the following independent components. These components are not relicensed under MisfoShiftTranscriber's license.

## whisper.cpp

- Project: whisper.cpp
- Recommended version: `1.9.0`
- Upstream: <https://github.com/ggml-org/whisper.cpp>
- Release artifact: `whisper-bin-x64.zip`
- Release URL: <https://github.com/ggml-org/whisper.cpp/releases/tag/v1.9.0>
- Artifact SHA-256: `00c4304b6be363a224a4b69829df49009f74131df8c3ce6a5878b89a11cd26ef`
- License: MIT
- License text: <https://github.com/ggml-org/whisper.cpp/blob/v1.9.0/LICENSE>

The application downloads the unmodified Windows x64 release archive directly from the upstream GitHub Release when requested by the user. It is not included in this source repository.

## Whisper ggml model files

- Model repository: <https://huggingface.co/ggerganov/whisper.cpp>
- Upstream model implementation: <https://github.com/openai/whisper>
- Repository license designation: MIT

The application can download the following unmodified model files directly from the model repository:

| File | Size | SHA-256 |
| --- | ---: | --- |
| `ggml-small.bin` | 487,601,967 bytes | `1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b` |
| `ggml-medium.bin` | 1,533,763,059 bytes | `6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208` |
| `ggml-large-v3.bin` | 3,095,033,483 bytes | `64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2` |

Model files are not included in this source repository or in the default application bundle. Users should review the upstream model repository and license information before redistribution.

## FFmpeg / BtbN FFmpeg Builds

MisfoShiftTranscriber can use an existing FFmpeg executable or download the pinned BtbN build below when requested by the user. The default application bundle does not include FFmpeg.

- FFmpeg project: <https://ffmpeg.org/>
- FFmpeg source: <https://ffmpeg.org/download.html>
- Build project: <https://github.com/BtbN/FFmpeg-Builds>
- Build release tag: `autobuild-2026-08-16-13-00`
- FFmpeg version/build identifier: `n8.1.2-44-g7c533d0f86-20260816`
- FFmpeg source revision: `7c533d0f86`
- BtbN FFmpeg-Builds commit: `590a6612d7d961e9258429e501619e0b7d7cbedf`
- Artifact: `ffmpeg-n8.1.2-44-g7c533d0f86-win64-lgpl-8.1.zip`
- Artifact size: `146,088,248` bytes
- Artifact SHA-256: `907a6bbc7aa100f5392309c5be4f527d12241121eda1c46db1c62b0054019db1`
- Build options include `--enable-version3`; `--enable-gpl` and `--enable-nonfree` are not enabled
- Binary license: GNU Lesser General Public License, version 3 or later (`LGPL-3.0-or-later`)
- LGPL text: <https://www.gnu.org/licenses/lgpl-3.0.html>
- Exact revision lookup: <https://git.ffmpeg.org/gitweb/ffmpeg.git/commit/7c533d0f86>

FFmpeg is executed as an external program. The application downloads the unmodified archive directly from the BtbN GitHub Release and verifies the pinned digest. Downloading a binary on a user's machine does not place it in this source repository.

If a future MisfoShiftTranscriber Release redistributes the FFmpeg binary, the distributor must comply with the LGPL. Preserve the upstream notices, include the applicable license text, and make the corresponding source and build information available in the manner required by the license. FFmpeg remains a separate executable invoked by the application and is not included in the default bundle. Do not substitute a build made with `--enable-gpl` or `--enable-nonfree` without a new license review.

This notice is an engineering record, not legal advice. Review license obligations before every public binary release.
