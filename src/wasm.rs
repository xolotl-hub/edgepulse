use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::engine::EdgePulseState;
use crate::model::RenderOutput;
use crate::scale;

#[wasm_bindgen]
pub struct EdgePulseCore {
    state: EdgePulseState,
    ctx: Option<CanvasRenderingContext2d>,
}

#[wasm_bindgen]
impl EdgePulseCore {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: Option<HtmlCanvasElement>) -> Result<EdgePulseCore, JsValue> {
        let ctx = match canvas {
            Some(c) => c
                .get_context("2d")?
                .and_then(|o| o.dyn_into::<CanvasRenderingContext2d>().ok()),
            None => None,
        };
        Ok(EdgePulseCore {
            state: EdgePulseState::new(),
            ctx,
        })
    }

    pub fn set_data_flat(&mut self, ohlcv: &[f64]) {
        self.state.set_data_flat(ohlcv);
    }

    pub fn update_candle(&mut self, t: f64, o: f64, h: f64, l: f64, c: f64, v: f64) {
        self.state.update_candle(t, o, h, l, c, v);
    }

    pub fn prepend_historical_flat(&mut self, ohlcv: &[f64]) -> usize {
        self.state.prepend_historical_flat(ohlcv)
    }

    pub fn set_overlays(&mut self, render_output_json: &str) -> Result<(), JsValue> {
        if render_output_json.is_empty() {
            self.state.set_overlays(None);
            return Ok(());
        }
        let parsed: RenderOutput = serde_json::from_str(render_output_json)
            .map_err(|e| JsValue::from_str(&format!("RenderOutput JSON inválido: {e}")))?;
        self.state.set_overlays(Some(parsed));
        Ok(())
    }

    pub fn toggle_indicator(&mut self, key: &str) -> bool {
        self.state.toggle_indicator(key)
    }

    pub fn set_log_scale(&mut self, enabled: bool) {
        self.state.viewport.log_scale = enabled;
    }

    pub fn resize(&mut self, width: f64, height: f64, dpr: f64) {
        self.state.resize(width, height);
        if let Some(ctx) = &self.ctx {
            let _ = ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);
        }
    }

    pub fn pan(&mut self, delta_px: f64) {
        self.state.pan(delta_px);
    }

    pub fn zoom_at(&mut self, x: f64, ratio: f64) {
        self.state.zoom_at(x, ratio);
    }

    pub fn set_visible_time_range(&mut self, from: f64, to: f64) {
        self.state.set_visible_time_range(from, to);
    }

    pub fn get_visible_time_range(&self) -> JsValue {
        match self.state.get_visible_time_range() {
            Some(r) => serde_wasm_bindgen::to_value(&r).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    pub fn get_visible_logical_range(&self) -> JsValue {
        let r = self.state.get_visible_logical_range();
        serde_wasm_bindgen::to_value(&r).unwrap_or(JsValue::NULL)
    }

    pub fn needs_more_history(&self) -> bool {
        self.state.needs_more_history()
    }

    pub fn coordinate_to_price(&self, y: f64) -> f64 {
        self.state.coordinate_to_price(y)
    }

    pub fn price_to_coordinate(&self, price: f64) -> f64 {
        self.state.price_to_coordinate(price)
    }

    pub fn coordinate_to_time(&self, x: f64) -> Option<f64> {
        self.state.coordinate_to_time(x)
    }

    pub fn time_to_coordinate(&self, time: f64) -> Option<f64> {
        self.state.time_to_coordinate(time)
    }

    pub fn candle_count(&self) -> usize {
        self.state.candles.len()
    }

    /// Proyecta y recorta overlays al viewport actual con anti-colisión de marcadores en Rust
    pub fn project_overlays(&self) -> JsValue {
        let projected = crate::overlays::project_overlays(&self.state);
        serde_wasm_bindgen::to_value(&projected).unwrap_or(JsValue::NULL)
    }

    /// Renderiza fondo, grilla, volumen, velas, EMAs y overlays directamente desde memoria lineal Rust
    pub fn render(&self) {
        let Some(ctx) = &self.ctx else {
            return;
        };
        let vp = &self.state.viewport;
        let plot_w = vp.plot_width();
        let plot_h = vp.plot_height();

        ctx.clear_rect(0.0, 0.0, vp.width, vp.height);
        ctx.set_fill_style_str("#0b0d10");
        ctx.fill_rect(0.0, 0.0, vp.width, vp.height);

        ctx.set_fill_style_str("#0d1017");
        ctx.fill_rect(plot_w, 0.0, vp.price_scale_width, vp.height);
        ctx.fill_rect(0.0, plot_h, plot_w, vp.time_scale_height);

        // Grilla de precios
        let ticks = scale::calculate_price_ticks(vp.visible_min_price, vp.visible_max_price, 7);
        ctx.set_stroke_style_str("#161a22");
        ctx.set_line_width(1.0);
        for p in ticks {
            let y = scale::price_to_y(vp, p).round() + 0.5;
            if y >= 0.0 && y <= plot_h {
                ctx.begin_path();
                ctx.move_to(0.0, y);
                ctx.line_to(plot_w, y);
                ctx.stroke();
            }
        }

        let n = self.state.candles.len();
        if n == 0 {
            return;
        }
        let (start, end, _, _) = scale::visible_index_range(vp, n);
        let candle_w = (vp.bar_spacing * 0.72).floor().max(1.0);

        // Volumen
        if self.state.indicators.volume && !self.state.volume.is_empty() {
            let mut max_vol = 0.0_f64;
            for i in start..=end {
                if let Some(v) = self.state.volume.get(i) {
                    max_vol = max_vol.max(v.value);
                }
            }
            if max_vol > 0.0 {
                let vol_h = plot_h * 0.15;
                for i in start..=end {
                    let c = self.state.candles[i];
                    let v = self.state.volume.get(i).map(|x| x.value).unwrap_or(0.0);
                    if v <= 0.0 {
                        continue;
                    }
                    let x = scale::index_to_x(vp, n, i as f64);
                    let h = (v / max_vol) * vol_h;
                    let y = plot_h - h;
                    ctx.set_fill_style_str(if c.close >= c.open {
                        "rgba(0, 230, 118, 0.32)"
                    } else {
                        "rgba(255, 51, 102, 0.32)"
                    });
                    ctx.fill_rect((x - candle_w / 2.0).round(), y.round(), candle_w, h.ceil());
                }
            }
        }

        // Velas japonesas
        for i in start..=end {
            let c = self.state.candles[i];
            let x = scale::index_to_x(vp, n, i as f64).round();
            let color = if c.close >= c.open {
                "#00e676"
            } else {
                "#ff3366"
            };
            let open_y = scale::price_to_y(vp, c.open).round();
            let close_y = scale::price_to_y(vp, c.close).round();
            let high_y = scale::price_to_y(vp, c.high).round();
            let low_y = scale::price_to_y(vp, c.low).round();

            ctx.set_fill_style_str(color);
            ctx.fill_rect(x, high_y, 1.0, (low_y - high_y).max(1.0));
            let body_top = open_y.min(close_y);
            let body_h = (close_y - open_y).abs().max(1.0);
            ctx.fill_rect((x - candle_w / 2.0).round(), body_top, candle_w, body_h);
        }

        // EMAs integradas
        if self.state.indicators.ema36 {
            Self::draw_series_line(ctx, vp, n, &self.state.ema36, start, end, "#00e5ff", 1.8);
        }
        if self.state.indicators.ema55 {
            Self::draw_series_line(ctx, vp, n, &self.state.ema55, start, end, "#ffb300", 1.8);
        }

        // Overlays de EdgeScript
        if let Some(overlays) = &self.state.overlays {
            for plot in &overlays.plots {
                ctx.save();
                ctx.set_stroke_style_str(&plot.color);
                ctx.set_line_width(plot.line_width.unwrap_or(1.8));
                ctx.begin_path();
                let mut started = false;
                for pt in &plot.data {
                    if let Some(val) = pt.value {
                        if let Some(x) = scale::time_to_x(vp, &self.state.candles, pt.time) {
                            let y = scale::price_to_y(vp, val);
                            if !started {
                                ctx.move_to(x, y);
                                started = true;
                            } else {
                                ctx.line_to(x, y);
                            }
                        }
                    }
                }
                ctx.stroke();
                ctx.restore();
            }
            for hl in &overlays.hlines {
                let y = scale::price_to_y(vp, hl.price).round() + 0.5;
                if y >= 0.0 && y <= plot_h {
                    ctx.save();
                    ctx.set_stroke_style_str(&hl.color);
                    ctx.set_line_width(1.2);
                    ctx.begin_path();
                    ctx.move_to(0.0, y);
                    ctx.line_to(plot_w, y);
                    ctx.stroke();
                    ctx.restore();
                }
            }
        }

        // Bordes separadores
        ctx.set_stroke_style_str("#1f242c");
        ctx.set_line_width(1.0);
        ctx.begin_path();
        ctx.move_to(plot_w + 0.5, 0.0);
        ctx.line_to(plot_w + 0.5, vp.height);
        ctx.move_to(0.0, plot_h + 0.5);
        ctx.line_to(vp.width, plot_h + 0.5);
        ctx.stroke();
    }

    fn draw_series_line(
        ctx: &CanvasRenderingContext2d,
        vp: &crate::model::Viewport,
        n: usize,
        series: &[Option<f64>],
        start: usize,
        end: usize,
        color: &str,
        width: f64,
    ) {
        if series.is_empty() {
            return;
        }
        ctx.save();
        ctx.set_stroke_style_str(color);
        ctx.set_line_width(width);
        ctx.begin_path();
        let mut started = false;
        let s = start.saturating_sub(1);
        let e = (end + 1).min(n - 1);
        for i in s..=e {
            if let Some(Some(val)) = series.get(i) {
                let x = scale::index_to_x(vp, n, i as f64);
                let y = scale::price_to_y(vp, *val);
                if !started {
                    ctx.move_to(x, y);
                    started = true;
                } else {
                    ctx.line_to(x, y);
                }
            }
        }
        ctx.stroke();
        ctx.restore();
    }
}
