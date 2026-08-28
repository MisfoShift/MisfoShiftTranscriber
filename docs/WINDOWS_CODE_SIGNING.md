# Windows code signing preparation

MisfoShiftTranscriber Windows artifacts are currently unsigned. This repository intentionally contains no signing certificate, private key, password, hardware-token credential, or cloud-signing token.

## Required items

- A Windows Authenticode code-signing certificate from a trusted CA, or an approved managed signing service.
- Access to the private key through the Windows certificate store, hardware token, HSM, or managed signing service.
- `signtool.exe` from the Windows SDK when using the local certificate store.
- An RFC 3161 timestamp service supported by the certificate provider.
- Protected CI secrets or workload identity if signing is automated.

## Tauri configuration points

Tauri 2 supports these fields under `bundle.windows` in `src-tauri/tauri.conf.json`:

- `certificateThumbprint`: SHA-1 thumbprint used to locate a certificate in the Windows certificate store.
- `digestAlgorithm`: use `sha256`.
- `timestampUrl`: timestamp endpoint supplied by the certificate provider.
- `tsp`: set according to whether that endpoint uses RFC 3161.
- `signCommand`: optional custom command for an HSM or managed signing service; `%1` is replaced with the binary path.

Do not commit a real thumbprint if organizational policy treats it as deployment metadata, and never place a private key or password in Tauri configuration. Prefer environment-protected CI configuration. Introduce either the certificate-store fields or `signCommand`, not both, and validate the exact behavior with the installed Tauri CLI version.

Example shape only:

```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "CERTIFICATE_THUMBPRINT_FROM_SECURE_CONFIGURATION",
      "digestAlgorithm": "sha256",
      "timestampUrl": "RFC3161_URL_FROM_CERTIFICATE_PROVIDER",
      "tsp": true
    }
  }
}
```

## Release flow

1. Build and test on a controlled Windows runner.
2. Let the Tauri bundler sign the application and installer artifacts using the chosen configuration.
3. Verify the executable, MSI, and NSIS installer signatures and timestamps.
4. Generate `SHA256SUMS.txt` only after signing, because signing changes file hashes.
5. Test installation and SmartScreen behavior on clean Windows 10 and Windows 11 machines.
6. Publish only the signed artifacts whose hashes match the final checksum list.

Verification commands:

```powershell
Get-AuthenticodeSignature -LiteralPath .\misfo-shift-transcriber.exe | Format-List
signtool verify /pa /all /v .\misfo-shift-transcriber.exe
signtool verify /pa /all /v .\MisfoShiftTranscriber.msi
signtool verify /pa /all /v .\MisfoShiftTranscriber-setup.exe
```

Confirm `Status` is `Valid`, the subject matches the intended publisher, the chain is trusted, the digest is SHA-256, and a valid timestamp is present. A certificate alone does not guarantee immediate SmartScreen reputation; record actual SmartScreen behavior in the release checklist.
