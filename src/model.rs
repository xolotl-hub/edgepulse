use serde::{Deserialize, Serialize};

/// Vela OHLC alineada con `apps/web/src/types/market.js`
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Candle {
    pub time: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VolumeBar {
    pub time: f64,
    pub value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VisibleRange {
    pub from: f64,
    pub to: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LogicalRange {
    pub from: i64,
    pub to: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IndicatorsState {
    pub ema36: bool,
    pub ema55: bool,
    pub volume: bool,
}

impl Default for IndicatorsState {
    fn default() -> Self {
        Self {
            ema36: true,
            ema55: true,
            volume: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotPoint {
    pub time: f64,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotSeries {
    pub id: String,
    pub title: String,
    pub color: String,
    #[serde(rename = "lineWidth")]
    pub line_width: Option<f64>,
    pub data: Vec<PlotPoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HLineOverlay {
    pub id: String,
    pub price: f64,
    pub title: String,
    pub color: String,
    pub style: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoxZoneOverlay {
    pub id: String,
    #[serde(rename = "timeStart")]
    pub time_start: f64,
    #[serde(rename = "timeEnd")]
    pub time_end: f64,
    #[serde(rename = "priceTop")]
    pub price_top: f64,
    #[serde(rename = "priceBottom")]
    pub price_bottom: f64,
    pub color: String,
    #[serde(rename = "fillColor")]
    pub fill_color: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkerOverlay {
    pub id: String,
    pub time: f64,
    pub price: f64,
    #[serde(rename = "type")]
    pub kind: String,
    pub label: String,
    pub color: String,
}

/// Salida canónica compartida con `edge-script-rs` y `ChartEnginePort.setOverlays`
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderOutput {
    #[serde(default)]
    pub plots: Vec<PlotSeries>,
    #[serde(default)]
    pub hlines: Vec<HLineOverlay>,
    #[serde(default)]
    pub boxes: Vec<BoxZoneOverlay>,
    #[serde(default)]
    pub markers: Vec<MarkerOverlay>,
    #[serde(default)]
    pub logs: Vec<String>,
    pub error: Option<String>,
}

/// Estado geométrico del viewport (equivalente a `ChartEngine` en JS)
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub width: f64,
    pub height: f64,
    pub price_scale_width: f64,
    pub time_scale_height: f64,
    pub bar_spacing: f64,
    pub min_bar_spacing: f64,
    pub max_bar_spacing: f64,
    pub right_offset: f64,
    pub scroll_offset: f64,
    pub visible_min_price: f64,
    pub visible_max_price: f64,
    pub is_auto_scale: bool,
    pub user_price_scale_multiplier: f64,
    pub user_price_offset: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 500.0,
            price_scale_width: 72.0,
            time_scale_height: 26.0,
            bar_spacing: 9.0,
            min_bar_spacing: 0.5,
            max_bar_spacing: 65.0,
            right_offset: 12.0,
            scroll_offset: 0.0,
            visible_min_price: 0.0,
            visible_max_price: 1.0,
            is_auto_scale: true,
            user_price_scale_multiplier: 1.0,
            user_price_offset: 0.0,
        }
    }
}

impl Viewport {
    pub fn plot_width(&self) -> f64 {
        (self.width - self.price_scale_width).max(50.0)
    }

    pub fn plot_height(&self) -> f64 {
        (self.height - self.time_scale_height).max(50.0)
    }
}
