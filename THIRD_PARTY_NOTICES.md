# Third-Party Notices

MisfoShiftTranscriber can download or use the following independent components. These components are not relicensed under MisfoShiftTranscriber's license.

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
- FFmpeg version/build identifier: `n8.1.2-44-g7c533d0f86-20260815`
- FFmpeg source revision: `7c533d0f86`
- BtbN FFmpeg-Builds commit: `590a6612d7d961e9258429e501619e0b7d7cbedf`
- Artifact: `ffmpeg-n8.1.2-44-g7c533d0f86-win64-gpl-8.1.zip`
- Artifact SHA-256: `d2425b12dc746a2b044148c6100440d4065876ac4ed6e3eb13a68437b7719796`
- Build options include `--enable-gpl --enable-version3`; `--enable-nonfree` is not enabled
- Binary license: GNU General Public License, version 3 or later (`GPL-3.0-or-later`)
- GPL text: <https://www.gnu.org/licenses/gpl-3.0.html>
- Exact revision lookup: <https://git.ffmpeg.org/gitweb/ffmpeg.git/commit/7c533d0f86>

FFmpeg is executed as an external program. The application downloads the unmodified archive directly from the BtbN GitHub Release and verifies the pinned digest. Downloading a binary on a user's machine does not place it in this source repository.

If a future MisfoShiftTranscriber Release redistributes the FFmpeg binary, the distributor must comply with the GPL. At minimum, include the unmodified license text and this notice, and provide the complete corresponding source for the exact FFmpeg revision, dependencies, and matching BtbN build definitions in the manner required by the license. A GitHub-generated `Source code (zip)` for BtbN alone is not the complete corresponding source for the produced FFmpeg binary. Do not distribute a build made with `--enable-nonfree`.

This notice is an engineering record, not legal advice. Review license obligations before every public binary release.
