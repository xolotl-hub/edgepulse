use crate::model::{Candle, Viewport, VolumeBar};

/// Parsea un buffer plano `[t, o, h, l, c, v, ...]` (stride = 6) en velas y volumen sin JSON
pub fn parse_flat_ohlcv(flat: &[f64]) -> (Vec<Candle>, Vec<VolumeBar>) {
    let count = flat.len() / 6;
    let mut candles = Vec::with_capacity(count);
    let mut volume = Vec::with_capacity(count);

    for chunk in flat.chunks_exact(6) {
        let (time, open, high, low, close, vol) =
            (chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5]);
        candles.push(Candle {
            time,
            open,
            high,
            low,
            close,
        });
        volume.push(VolumeBar {
            time,
            value: vol,
            color: None,
        });
    }

    (candles, volume)
}

pub fn index_to_x(vp: &Viewport, total_candles: usize, index: f64) -> f64 {
    if total_candles == 0 {
        return 0.0;
    }
    let last_idx = (total_candles - 1) as f64;
    vp.plot_width() - ((last_idx - index + vp.right_offset) - vp.scroll_offset) * vp.bar_spacing
}

pub fn x_to_index(vp: &Viewport, total_candles: usize, x: f64) -> f64 {
    if total_candles == 0 {
        return 0.0;
    }
    let last_idx = (total_candles - 1) as f64;
    (last_idx + vp.right_offset - vp.scroll_offset) - (vp.plot_width() - x) / vp.bar_spacing
}

pub fn estimate_time_step(candles: &[Candle]) -> f64 {
    if candles.len() >= 2 {
        let diff = (candles[1].time - candles[0].time).abs();
        if diff > 0.0 {
            return diff;
        }
    }
    3600.0
}

pub fn find_closest_index_by_time(candles: &[Candle], time: f64) -> usize {
    if candles.is_empty() {
        return 0;
    }
    let idx = candles.partition_point(|c| c.time < time);
    idx.min(candles.len() - 1)
}

pub fn time_to_x(vp: &Viewport, candles: &[Candle], time: f64) -> Option<f64> {
    if candles.is_empty() {
        return None;
    }
    let n = candles.len();
    let first = candles[0].time;
    let last = candles[n - 1].time;
    let step = estimate_time_step(candles);

    if time < first {
        let bars_before = (first - time) / step;
        return Some(index_to_x(vp, n, -bars_before));
    }
    if time > last {
        let bars_after = (time - last) / step;
        return Some(index_to_x(vp, n, (n - 1) as f64 + bars_after));
    }

    let idx = find_closest_index_by_time(candles, time);
    Some(index_to_x(vp, n, idx as f64))
}

pub fn x_to_time(vp: &Viewport, candles: &[Candle], x: f64) -> Option<f64> {
    if candles.is_empty() {
        return None;
    }
    let round_idx = x_to_index(vp, candles.len(), x).round() as i64;
    let n = candles.len() as i64;

    if round_idx >= 0 && round_idx < n {
        return Some(candles[round_idx as usize].time);
    }

    let step = estimate_time_step(candles);
    if round_idx < 0 {
        Some(candles[0].time + (round_idx as f64) * step)
    } else {
        let last = candles[candles.len() - 1].time;
        Some(last + ((round_idx - (n - 1)) as f64) * step)
    }
}

pub fn price_to_y(vp: &Viewport, price: f64) -> f64 {
    let range = vp.visible_max_price - vp.visible_min_price;
    let plot_h = vp.plot_height();
    if range <= 0.0 {
        return plot_h / 2.0;
    }
    plot_h - ((price - vp.visible_min_price) / range) * plot_h
}

pub fn y_to_price(vp: &Viewport, y: f64) -> f64 {
    let range = vp.visible_max_price - vp.visible_min_price;
    let plot_h = vp.plot_height();
    if range <= 0.0 {
        return 0.0;
    }
    vp.visible_min_price + ((plot_h - y) / plot_h) * range
}

pub fn visible_index_range(vp: &Viewport, total_candles: usize) -> (usize, usize, i64, i64) {
    if total_candles == 0 {
        return (0, 0, 0, 0);
    }
    let raw_start = x_to_index(vp, total_candles, 0.0).floor() as i64;
    let raw_end = x_to_index(vp, total_candles, vp.plot_width()).ceil() as i64;
    let max_idx = (total_candles - 1) as i64;

    let start = raw_start.clamp(0, max_idx) as usize;
    let end = raw_end.clamp(0, max_idx) as usize;
    (start, end, raw_start, raw_end)
}

/// Calcula EMA alineada 1:1 con `candles` preservando precisión f64 completa (sub-centavo)
pub fn calculate_ema(candles: &[Candle], period: usize) -> Vec<Option<f64>> {
    if candles.is_empty() || period == 0 {
        return Vec::new();
    }
    if candles.len() < period {
        return vec![None; candles.len()];
    }

    let mut out = Vec::with_capacity(candles.len());
    let k = 2.0 / (period as f64 + 1.0);
    let mut sum = 0.0;

    for candle in candles.iter().take(period - 1) {
        sum += candle.close;
        out.push(None);
    }

    sum += candles[period - 1].close;
    let mut ema = sum / (period as f64);
    out.push(Some(ema));

    for candle in candles.iter().skip(period) {
        ema = candle.close * k + ema * (1.0 - k);
        out.push(Some(ema));
    }

    out
}

/// Genera ticks limpios (1-2-5) para el eje Y
pub fn calculate_price_ticks(min: f64, max: f64, target_count: usize) -> Vec<f64> {
    let range = max - min;
    if range <= 0.0 || !range.is_finite() || target_count == 0 {
        return Vec::new();
    }

    let rough_step = range / target_count as f64;
    let step_power = 10f64.powf(rough_step.log10().floor());
    let ratio = rough_step / step_power;
    let nice = if ratio >= 5.0 {
        5.0
    } else if ratio >= 2.0 {
        2.0
    } else {
        1.0
    };
    let step = nice * step_power;
    if step <= 0.0 || !step.is_finite() {
        return Vec::new();
    }

    let mut ticks = Vec::new();
    let mut p = (min / step).ceil() * step;
    while p <= max && ticks.len() < 128 {
        ticks.push(p);
        p += step;
    }
    ticks
}
