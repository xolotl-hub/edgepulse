# ⚡ EdgePulse Core (`edge-pulse-core`)

Punto de anclaje (Git Submodule o crate local) para el motor gráfico en **Rust / WebAssembly** de **EdgeTrade**:

* **Nombre del Crate:** `edge-pulse-core`
* **Tecnología:** Rust + `wasm-bindgen` + `web-sys` (Canvas 2D / WebGL)
* **Objetivo:** Proyección gráfica de alto rendimiento a 60 FPS, cálculo hiper-eficiente de coordenadas y manejo en memoria lineal de cientos de miles de velas sin sobrecargar el recolector de basura de JavaScript.

---

## 🔌 Contrato Wasm con el Frontend

El core gráfico en Rust expone la clase Wasm coincidente con el puerto `ChartEnginePort` de EdgeTrade:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct EdgePulseCore {
    // Memoria lineal y buffers de renderizado en Rust
}

#[wasm_bindgen]
impl EdgePulseCore {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: web_sys::HtmlCanvasElement, options_json: &str) -> Self;

    pub fn set_data(&mut self, candles_json: &str, volume_json: &str);
    pub fn update_candle(&mut self, candle_json: &str, volume_json: &str);
    pub fn prepend_historical_data(&mut self, candles_json: &str, volume_json: &str);
    pub fn set_overlays(&mut self, render_output_json: &str);
    pub fn resize(&mut self, width: f64, height: f64);
    pub fn coordinate_to_price(&self, y: f64) -> f64;
    pub fn price_to_coordinate(&self, price: f64) -> f64;
    pub fn coordinate_to_time(&self, x: f64) -> f64;
    pub fn time_to_coordinate(&self, time: f64) -> f64;
    pub fn render(&mut self);
}
```

---

## 🛠️ Flujo de Compilación y Enlace

```bash
# 1. Compilar paquete Wasm para navegador
cd packages/edge-pulse/repo
wasm-pack build --target web --out-dir ../pkg

# 2. Conexión en el Frontend:
# El adaptador `src/adapters/WasmChartAdapter.js` importará:
# import init, { EdgePulseCore } from '../../packages/edge-pulse/pkg/edge_pulse_core.js';
```
