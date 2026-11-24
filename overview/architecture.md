# 🏛️ Arquitectura — `edge-pulse-core` (`Bridge` Rust/Wasm)

> Actualizado: 2026-11-24 (`$edgepulse:archi`) | Estado: `🟢 CONGRUENTE` (5 tests `cargo test` OK + Wasm 152 KB en `apps/web/wasm/edge-pulse/`)

```mermaid
graph LR
    MOD["src/model.rs\n(Candle, Viewport, RenderOutput)"] --> SCL["src/scale.rs\n(parse_flat_ohlcv, coords, EMA, ticks)"]
    SCL --> ENG["src/engine.rs\n(EdgePulseState)"]
    ENG --> WASM["src/wasm.rs\n(#[wasm_bindgen] EdgePulseCore + Canvas2D)"]
    WASM -->|wasm-pack --features wasm| PKG["apps/web/wasm/edge-pulse/\n(152 KB Wasm)"]
    PKG -->|WasmChartAdapter.js| PORT["ChartEnginePort (EdgeTrade)"]
```
