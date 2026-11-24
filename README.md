# ⚡ EdgePulse Core (`edge-pulse-core`)

Motor gráfico financiero en **Rust / WebAssembly** para **EdgeTrade** (`ChartEnginePort`):

* **Crate:** `edge-pulse-core` (`crate-type = ["cdylib", "rlib"]`)
* **Tecnología:** Rust + `serde` + feature `wasm` (`wasm-bindgen`, `serde-wasm-bindgen`, `web-sys`)
* **Arquitectura interna:**
  * `src/model.rs`: `Candle`, `VolumeBar`, `VisibleRange`, `LogicalRange`, `RenderOutput`, `Viewport`
  * `src/scale.rs`: `parse_flat_ohlcv(&[f64])`, transformaciones `index ↔ x`, `time ↔ x`, `price ↔ y`, `calculate_ema`, ticks `1-2-5`
  * `src/engine.rs`: `EdgePulseState` (gestión de velas, ticks en vivo sin JSON, paginación histórica, pan/zoom centrado en cursor)

---

## 🔌 Contrato de Frontera Wasm (Zero-JSON en Hot Path)

- Carga inicial / histórica: `set_data_flat(&mut self, ohlcv: &[f64])` y `prepend_historical_flat(&mut self, ohlcv: &[f64])` (buffer plano `[t, o, h, l, c, v, ...]`).
- Ticks en vivo: `update_candle(&mut self, t: f64, o: f64, h: f64, l: f64, c: f64, v: f64)`.
- Overlays de `edge-script-rs`: `set_overlays(&mut self, render_output_json: &str)` (`RenderOutput`).

---

## 🛠️ Tests Nativos y Compilación Wasm

```bash
# 1. Tests unitarios nativos (sin navegador)
cargo test

# 2. Compilar paquete Wasm para navegador (requiere target wasm32-unknown-unknown + wasm-pack)
wasm-pack build --target web --out-dir ./pkg -- --features wasm
```
