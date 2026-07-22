Locale seed directories:

- `zh-CN/` - default active locale
- `en-US/`, `ja-JP/`, `de-DE/`, `fr-FR/`, `ru-RU/`, `ko-KR/` - reserved locale directories

Locale seed files are optional. When present, their explicit execution order, version, and checksum are
owned by `seeds/seed.manifest.json`; empty locale sets use the SHA-256 digest of empty content.
