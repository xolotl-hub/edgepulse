pub mod engine;
pub mod model;
pub mod scale;

pub use engine::EdgePulseState;
pub use model::{
    BoxZoneOverlay, Candle, HLineOverlay, IndicatorsState, LogicalRange, MarkerOverlay, PlotPoint,
    PlotSeries, RenderOutput, Viewport, VisibleRange, VolumeBar,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_flat_candles(n: usize, base_price: f64) -> Vec<f64> {
        let mut flat = Vec::with_capacity(n * 6);
        for i in 0..n {
            let t = 1_700_000_000.0 + (i as f64) * 3600.0;
            let o = base_price + (i as f64) * base_price * 0.001;
            let c = o * 1.002;
            let h = c * 1.005;
            let l = o * 0.995;
            let v = 1000.0 + i as f64;
            flat.extend_from_slice(&[t, o, h, l, c, v]);
        }
        flat
    }

    #[test]
    fn flat_ohlcv_and_subcent_ema_precision() {
        let mut state = EdgePulseState::new();
        let flat = sample_flat_candles(60, 0.00001234);
        state.set_data_flat(&flat);

        assert_eq!(state.candles.len(), 60);
        let ema36_last = state.ema36[59].expect("EMA 36 must exist at index 59");
        assert!(ema36_last > 0.000012, "EMA truncated: {ema36_last}");
        assert!(state.ema36[34].is_none());
    }

    #[test]
    fn price_and_time_coordinate_roundtrip() {
        let mut state = EdgePulseState::new();
        state.resize(1000.0, 600.0);
        state.set_data_flat(&sample_flat_candles(100, 50_000.0));

        let mid_price = state.candles[80].close;
        let y = state.price_to_coordinate(mid_price);
        let back_price = state.coordinate_to_price(y);
        assert!((back_price - mid_price).abs() < 1e-6);

        let t = state.candles[80].time;
        let x = state.time_to_coordinate(t).expect("x coordinate");
        let back_t = state.coordinate_to_time(x).expect("time coordinate");
        assert!((back_t - t).abs() < 1e-6);
    }

    #[test]
    fn prepend_historical_deduplicates_and_updates_len() {
        let mut state = EdgePulseState::new();
        let all = sample_flat_candles(40, 100.0);
        state.set_data_flat(&all[20 * 6..]);
        assert_eq!(state.candles.len(), 20);

        // Include overlapping candles to verify deduplication
        let added = state.prepend_historical_flat(&all[..25 * 6]);
        assert_eq!(added, 20);
        assert_eq!(state.candles.len(), 40);
        assert!(state.candles[0].time < state.candles[39].time);
    }

    #[test]
    fn zoom_keeps_anchor_bar_under_cursor() {
        let mut state = EdgePulseState::new();
        state.resize(900.0, 500.0);
        state.set_data_flat(&sample_flat_candles(120, 200.0));

        let cursor_x = 400.0;
        let bar_before = scale::x_to_index(&state.viewport, state.candles.len(), cursor_x);
        state.zoom_at(cursor_x, 1.5);
        let bar_after = scale::x_to_index(&state.viewport, state.candles.len(), cursor_x);
        assert!((bar_before - bar_after).abs() < 1e-6);
    }

    #[test]
    fn toggle_indicator_and_overlays_serde() {
        let mut state = EdgePulseState::new();
        assert!(!state.toggle_indicator("ema36"));
        assert!(state.toggle_indicator("ema36"));

        let json = r##"{"plots":[],"hlines":[{"id":"hl_1","price":100.5,"title":"S/R","color":"#f59e0b","style":"dashed"}],"boxes":[],"markers":[]}"##;
        let overlays: RenderOutput = serde_json::from_str(json).expect("valid RenderOutput JSON");
        assert_eq!(overlays.hlines.len(), 1);
        state.set_overlays(Some(overlays));
        assert!(state.overlays.is_some());
    }
}
