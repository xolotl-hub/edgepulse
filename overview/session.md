# Estado de Sesión — `edge-pulse-core` (Bridge Rust/Wasm)

Agente: Antigravity (Gemini 4 Neon) — 2026-11-24
Agente que reanuda:
Modo: `laconico`
Rol: `Bridge` (`rust-agent-rules`, Spoke Federado de `edgetrade`)
Validación: `cargo test` 5/5 OK (`cargo clippy` omitida: componente no instalado)

## Reanudar
- Instalar `wasm32-unknown-unknown` + `wasm-pack`, implementar `src/renderer.rs` (`web-sys` Canvas 2D) + bindings `#[wasm_bindgen]` y `apps/web/src/adapters/WasmChartAdapter.js`.

## Cambios
- [2026-11-24]: Implementado núcleo Rust puro `edge-pulse-core` (`Cargo.toml`, `src/{model,scale,engine,lib}.rs`, 5 tests pasando) con buffer plano `&[f64]` zero-JSON; detectado y corregido bug de signo en `zoom_at` (también en `apps/web/src/chart.js`).
