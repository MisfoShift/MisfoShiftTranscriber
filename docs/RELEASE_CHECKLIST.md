# Public release manual checklist

## Release record

| Field | Value |
| --- | --- |
| Version / commit | |
| Test date | |
| Tester | |
| MSI file / SHA-256 | |
| NSIS file / SHA-256 | |
| Windows 10 edition / build | |
| Windows 11 edition / build | |

Use a clean VM or PC for each OS. Record `OK`, `NG`, or `N/A` and add evidence or an issue link in the notes column.

| Check | Windows 10 | Windows 11 | Notes / issue |
| --- | --- | --- | --- |
| Git history author names and email addresses are approved for public disclosure | [ ] | [ ] | |
| Tracked files contain no credentials, local absolute paths, or unintended binary artifacts | [ ] | [ ] | |
| Installer SHA-256 matches `SHA256SUMS.txt` | [ ] | [ ] | |
| MSI installation completes | [ ] | [ ] | |
| NSIS installation completes | [ ] | [ ] | |
| Installer license screen shows the MIT `LICENSE` where supported | [ ] | [ ] | |
| Installed files include `LICENSE` and `THIRD_PARTY_NOTICES.md` | [ ] | [ ] | |
| Installed files do not include whisper.cpp, models, or FFmpeg | [ ] | [ ] | |
| First launch completes without an application crash | [ ] | [ ] | |
| Missing components show the initial setup guidance | [ ] | [ ] | |
| Recommended setup completes in order | [ ] | [ ] | |
| whisper.cpp recommended build downloads and verifies | [ ] | [ ] | |
| Whisper small model downloads and becomes active | [ ] | [ ] | |
| FFmpeg LGPL build downloads and is detected | [ ] | [ ] | |
| Microphone list and manual microphone selection work | [ ] | [ ] | |
| Microphone transcription starts, shows interim text, and stops | [ ] | [ ] | |
| Audio file transcription works for WAV without FFmpeg when applicable | [ ] | [ ] | |
| Audio/video format requiring FFmpeg is transcribed | [ ] | [ ] | |
| Copy, TXT save, edit, and clear work | [ ] | [ ] | |
| Manual whisper.cpp / model / FFmpeg selection still works | [ ] | [ ] | |
| App restart retains component and language settings | [ ] | [ ] | |
| With the network disconnected, microphone transcription works | [ ] | [ ] | |
| With the network disconnected, installed-file transcription works | [ ] | [ ] | |
| Offline use makes no unexpected external request | [ ] | [ ] | |
| Medium model downloads, installs, and transcribes | [ ] | [ ] | |
| Large model downloads, installs, and transcribes on a suitable PC | [ ] | [ ] | |
| Switching small / medium / large changes the active model | [ ] | [ ] | |
| Re-acquiring whisper.cpp preserves a usable installation | [ ] | [ ] | |
| Re-acquiring FFmpeg preserves a usable installation | [ ] | [ ] | |
| A failed or interrupted download leaves no `.part`, `.download`, or `.install-*` residue | [ ] | [ ] | |
| An oversized or corrupt response is rejected without replacing the installed component | [ ] | [ ] | |
| Uninstall completes | [ ] | [ ] | |
| Remaining `%APPDATA%\com.misfoshift.transcriber` files are recorded and expected | [ ] | [ ] | |
| Reinstall after uninstall works | [ ] | [ ] | |
| SmartScreen message and publisher display are recorded | [ ] | [ ] | |
| `Get-AuthenticodeSignature` result is recorded (expected `NotSigned` until signing is introduced) | [ ] | [ ] | |

## Final decision

| Field | Value |
| --- | --- |
| Blocking issues | |
| Accepted risks and owner | |
| Release approved by / date | |
| GitHub Release URL | |
