# Burn segment

Burn-down state derived from a cache file.

- Sampling + linear regression slope over `burn_lookback`.
- States: warming / idle / active; urgency colors.
- 5h/7d fallback windows; cache file via `CLAUDEBAR_BURN_FILE` override.
- Emits nothing when no cache file.
- Source: `src/segment/burn.rs`; see cross-session-state for the TSV layout.
- Tests: regression, state classification, urgency colors, missing-file hide.