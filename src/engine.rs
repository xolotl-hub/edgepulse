use crate::model::{
    Candle, IndicatorsState, LogicalRange, RenderOutput, Viewport, VisibleRange, VolumeBar,
};
use crate::scale;

/// Núcleo de estado y proyección geométrica de `EdgePulseCore` (independiente de DOM/Canvas)
#[derive(Debug, Default)]
pub struct EdgePulseState {
    pub candles: Vec<Candle>,
    pub volume: Vec<VolumeBar>,
    pub viewport: Viewport,
    pub indicators: IndicatorsState,
    pub ema36: Vec<Option<f64>>,
    pub ema55: Vec<Option<f64>>,
    pub overlays: Option<RenderOutput>,
    pub has_more_history: bool,
}

impl EdgePulseState {
    pub fn new() -> Self {
        Self {
            has_more_history: true,
            ..Self::default()
        }
    }

    pub fn set_data_flat(&mut self, ohlcv: &[f64]) {
        let (candles, volume) = scale::parse_flat_ohlcv(ohlcv);
        self.set_data(candles, volume);
    }

    pub fn set_data(&mut self, candles: Vec<Candle>, volume: Vec<VolumeBar>) {
        if candles.is_empty() {
            return;
        }
        self.candles = candles;
        self.volume = volume;
        self.has_more_history = true;
        self.viewport.scroll_offset = 0.0;
        self.viewport.is_auto_scale = true;
        self.viewport.user_price_scale_multiplier = 1.0;
        self.viewport.user_price_offset = 0.0;
        self.recalculate_emas();
        self.recalculate_visible_price_range();
    }

    pub fn update_candle(&mut self, time: f64, open: f64, high: f64, low: f64, close: f64, vol: f64) {
        let candle = Candle {
            time,
            open,
            high,
            low,
            close,
        };
        let vbar = VolumeBar {
            time,
            value: vol,
            color: None,
        };

        if let Some(last) = self.candles.last_mut() {
            if (last.time - time).abs() < f64::EPSILON {
                *last = candle;
            } else if time > last.time {
                self.candles.push(candle);
                if self.viewport.scroll_offset > 0.0 {
                    self.viewport.scroll_offset += 1.0;
                }
            }
        } else {
            self.candles.push(candle);
        }

        if let Some(last_v) = self.volume.last_mut() {
            if (last_v.time - time).abs() < f64::EPSILON {
                *last_v = vbar;
            } else if time > last_v.time {
                self.volume.push(vbar);
            }
        } else {
            self.volume.push(vbar);
        }

        self.recalculate_emas();
        self.recalculate_visible_price_range();
    }

    pub fn prepend_historical_flat(&mut self, ohlcv: &[f64]) -> usize {
        let (older_c, older_v) = scale::parse_flat_ohlcv(ohlcv);
        self.prepend_historical(older_c, older_v)
    }

    pub fn prepend_historical(
        &mut self,
        older_candles: Vec<Candle>,
        older_volume: Vec<VolumeBar>,
    ) -> usize {
        let Some(first_time) = self.candles.first().map(|c| c.time) else {
            self.set_data(older_candles, older_volume);
            return self.candles.len();
        };

        let mut unique_c: Vec<Candle> = older_candles
            .into_iter()
            .filter(|c| c.time < first_time)
            .collect();
        let added = unique_c.len();
        if added == 0 {
            self.has_more_history = false;
            return 0;
        }

        let mut unique_v: Vec<VolumeBar> = older_volume
            .into_iter()
            .filter(|v| v.time < first_time)
            .collect();

        unique_c.append(&mut self.candles);
        unique_v.append(&mut self.volume);
        self.candles = unique_c;
        self.volume = unique_v;

        self.recalculate_emas();
        self.recalculate_visible_price_range();
        added
    }

    pub fn set_overlays(&mut self, overlays: Option<RenderOutput>) {
        self.overlays = overlays;
    }

    pub fn toggle_indicator(&mut self, key: &str) -> bool {
        let state = match key {
            "ema36" => {
                self.indicators.ema36 = !self.indicators.ema36;
                self.recalculate_emas();
                self.indicators.ema36
            }
            "ema55" => {
                self.indicators.ema55 = !self.indicators.ema55;
                self.recalculate_emas();
                self.indicators.ema55
            }
            "volume" => {
                self.indicators.volume = !self.indicators.volume;
                self.indicators.volume
            }
            _ => false,
        };
        state
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        self.viewport.width = width.max(100.0);
        self.viewport.height = height.max(100.0);
        self.recalculate_visible_price_range();
    }

    pub fn pan(&mut self, delta_px: f64) {
        self.viewport.scroll_offset += delta_px / self.viewport.bar_spacing;
        self.recalculate_visible_price_range();
    }

    pub fn zoom_at(&mut self, x: f64, ratio: f64) {
        let old_spacing = self.viewport.bar_spacing;
        let new_spacing =
            (old_spacing * ratio).clamp(self.viewport.min_bar_spacing, self.viewport.max_bar_spacing);
        if (new_spacing - old_spacing).abs() < f64::EPSILON {
            return;
        }

        let bar_before = scale::x_to_index(&self.viewport, self.candles.len(), x);
        self.viewport.bar_spacing = new_spacing;
        let bar_after = scale::x_to_index(&self.viewport, self.candles.len(), x);
        self.viewport.scroll_offset += bar_after - bar_before;
        self.recalculate_visible_price_range();
    }

    pub fn get_visible_time_range(&self) -> Option<VisibleRange> {
        if self.candles.is_empty() {
            return None;
        }
        let (start, end, _, _) = scale::visible_index_range(&self.viewport, self.candles.len());
        Some(VisibleRange {
            from: self.candles[start].time,
            to: self.candles[end].time,
        })
    }

    pub fn set_visible_time_range(&mut self, from: f64, to: f64) {
        if self.candles.is_empty() || to <= from {
            return;
        }
        let idx_from = scale::find_closest_index_by_time(&self.candles, from);
        let idx_to = scale::find_closest_index_by_time(&self.candles, to);
        let bars = idx_to.saturating_sub(idx_from).max(1) as f64;

        self.viewport.bar_spacing = (self.viewport.plot_width() / bars)
            .clamp(self.viewport.min_bar_spacing, self.viewport.max_bar_spacing);
        let n = self.candles.len() as f64;
        self.viewport.scroll_offset = (n - 1.0 + self.viewport.right_offset) - idx_to as f64;
        self.recalculate_visible_price_range();
    }

    pub fn get_visible_logical_range(&self) -> LogicalRange {
        let (_, _, raw_start, raw_end) =
            scale::visible_index_range(&self.viewport, self.candles.len());
        LogicalRange {
            from: raw_start,
            to: raw_end,
        }
    }

    pub fn needs_more_history(&self) -> bool {
        self.has_more_history && !self.candles.is_empty() && self.get_visible_logical_range().from < 40
    }

    pub fn coordinate_to_price(&self, y: f64) -> f64 {
        scale::y_to_price(&self.viewport, y)
    }

    pub fn price_to_coordinate(&self, price: f64) -> f64 {
        scale::price_to_y(&self.viewport, price)
    }

    pub fn coordinate_to_time(&self, x: f64) -> Option<f64> {
        scale::x_to_time(&self.viewport, &self.candles, x)
    }

    pub fn time_to_coordinate(&self, time: f64) -> Option<f64> {
        scale::time_to_x(&self.viewport, &self.candles, time)
    }

    fn recalculate_emas(&mut self) {
        self.ema36 = if self.indicators.ema36 {
            scale::calculate_ema(&self.candles, 36)
        } else {
            Vec::new()
        };
        self.ema55 = if self.indicators.ema55 {
            scale::calculate_ema(&self.candles, 55)
        } else {
            Vec::new()
        };
    }

    pub fn recalculate_visible_price_range(&mut self) {
        if self.candles.is_empty() {
            self.viewport.visible_min_price = 0.0;
            self.viewport.visible_max_price = 100.0;
            return;
        }
        let (start, end, _, _) = scale::visible_index_range(&self.viewport, self.candles.len());
        let mut min_p = f64::INFINITY;
        let mut max_p = f64::NEG_INFINITY;

        for c in &self.candles[start..=end] {
            if c.low < min_p {
                min_p = c.low;
            }
            if c.high > max_p {
                max_p = c.high;
            }
        }

        if !min_p.is_finite() || !max_p.is_finite() {
            min_p = 0.0;
            max_p = 100.0;
        }
        if (max_p - min_p).abs() < f64::EPSILON {
            min_p *= 0.98;
            max_p *= 1.02;
        }

        let span = max_p - min_p;
        let base_min = min_p - span * 0.18;
        let base_max = max_p + span * 0.08;

        if self.viewport.is_auto_scale {
            self.viewport.visible_min_price = base_min;
            self.viewport.visible_max_price = base_max;
        } else {
            let center = (base_min + base_max) / 2.0 + self.viewport.user_price_offset;
            let half = ((base_max - base_min) / 2.0) * self.viewport.user_price_scale_multiplier;
            self.viewport.visible_min_price = center - half;
            self.viewport.visible_max_price = center + half;
        }
    }
}
