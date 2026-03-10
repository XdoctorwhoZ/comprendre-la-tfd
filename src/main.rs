mod dft;

use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};

use dft::{
    compute_dft, generate_plot_signal, sample_signal, SinusConfig,
};

// ─── Constants ───────────────────────────────────────────────────────────────

/// Maximum number of simultaneous sinusoidal components
const MAX_SINUS: usize = 5;

/// SVG viewport dimensions used for all charts
const SVG_W: f64 = 800.0;
const SVG_H: f64 = 200.0;

/// Chart margins (pixels inside the SVG viewport)
const M_L: f64 = 55.0; // left  – room for y-axis labels
const M_R: f64 = 10.0; // right
const M_T: f64 = 10.0; // top
const M_B: f64 = 28.0; // bottom – room for x-axis labels

// ─── Entry point ─────────────────────────────────────────────────────────────

fn main() {
    let cfg = Config::new().with_window(
        WindowBuilder::new()
            .with_title("Comprendre la TFD – Rust / Dioxus")
            .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0_f64, 800.0_f64)),
    );

    LaunchBuilder::new().with_cfg(cfg).launch(App);
}

// ─── SVG helpers ─────────────────────────────────────────────────────────────

/// Map data-space coordinates to SVG-pixel coordinates.
/// Returns (svg_x, svg_y) clamped to the plot area.
fn to_svg(
    xi: f64,
    yi: f64,
    x_min: f64,
    x_range: f64,
    y_min: f64,
    y_range: f64,
) -> (f64, f64) {
    let pw = SVG_W - M_L - M_R;
    let ph = SVG_H - M_T - M_B;
    let sx = M_L + (xi - x_min) / x_range * pw;
    // SVG y-axis is inverted
    let sy = SVG_H - M_B - (yi - y_min) / y_range * ph;
    (sx, sy)
}

/// Format a floating-point number compactly (≤ 4 significant figures).
fn fmt_f(v: f64) -> String {
    if v == 0.0 {
        return "0".to_string();
    }
    if v.abs() < 0.001 || v.abs() >= 1e5 {
        format!("{v:.2e}")
    } else if v.abs() < 1.0 {
        format!("{v:.4}")
    } else {
        format!("{v:.2}")
    }
}

// ─── SVG axis helpers ─────────────────────────────────────────────────────────

/// Compute `ticks` evenly-spaced values spanning [min, max].
fn axis_ticks(min: f64, max: f64, ticks: usize) -> Vec<f64> {
    if ticks < 2 {
        return vec![min, max];
    }
    (0..ticks)
        .map(|i| min + (max - min) * i as f64 / (ticks - 1) as f64)
        .collect()
}

// ─── App component ────────────────────────────────────────────────────────────

#[component]
fn App() -> Element {
    // ── reactive state ──────────────────────────────────────────────────────
    let mut nb_sinus = use_signal(|| 1_usize);
    let mut sinus_configs =
        use_signal(|| (0..MAX_SINUS).map(|_| SinusConfig::default()).collect::<Vec<_>>());
    let mut fm = use_signal(|| 1.0_f64); // measurement frequency
    let mut fe = use_signal(|| 2.5_f64); // sampling frequency (= 2.5 × Fm)
    let mut n_points = use_signal(|| 512_usize); // number of samples N

    // ── derived values (computed fresh each render) ─────────────────────────
    let nb = nb_sinus();
    let fe_val = fe();
    let n = n_points();
    let sa = if fe_val > 1e-10 { fe_val / n as f64 } else { 0.0 };

    let active_configs: Vec<SinusConfig> = {
        let guard = sinus_configs.read();
        guard[..nb].to_vec()
    };

    // Signal plot (1024 continuous points, 3 periods)
    let (sig_x, sig_y) = generate_plot_signal(&active_configs, 1024);

    // Sample plot
    let t_step = if fe_val > 1e-10 { 1.0 / fe_val } else { 0.0 };
    let samp_x: Vec<f64> = (0..n).map(|i| i as f64 * t_step).collect();
    let samp_y = sample_signal(&active_configs, n, fe_val);

    // DFT magnitude spectrum
    let dft_mag = compute_dft(&samp_y);
    let dft_x: Vec<f64> = (0..n).map(|k| k as f64 * sa).collect();

    // ── layout ──────────────────────────────────────────────────────────────
    rsx! {
        style { {CSS} }

        div { class: "app",
            // ── Left panel: controls ───────────────────────────────────────
            div { class: "panel-left",
                h2 { class: "panel-title", "Comprendre la TFD" }

                // ── Sinusoïde count ────────────────────────────────────────
                div { class: "section-title", "Sinusoïdes" }
                div { class: "form-row",
                    label { r#for: "nb-sinus", "Nombre :" }
                    select {
                        id: "nb-sinus",
                        value: "{nb_sinus}",
                        onchange: move |e| {
                            if let Ok(v) = e.value().parse::<usize>() {
                                nb_sinus.set(v.clamp(1, MAX_SINUS));
                            }
                        },
                        for i in 1..=MAX_SINUS {
                            option { value: "{i}", selected: i == nb, "{i}" }
                        }
                    }
                }

                // ── Per-sine configuration rows ────────────────────────────
                for idx in 0..MAX_SINUS {
                    SinusConfigRow {
                        key: "{idx}",
                        index: idx,
                        enabled: idx < nb,
                        config: {
                            let g = sinus_configs.read();
                            g[idx].clone()
                        },
                        on_change: move |new_cfg: SinusConfig| {
                            sinus_configs.with_mut(|v| v[idx] = new_cfg);
                        },
                    }
                }

                // ── Sampling parameters ────────────────────────────────────
                div { class: "section-title", "Paramètres" }

                div { class: "form-row",
                    label { r#for: "fm", "Fm =" }
                    input {
                        id: "fm",
                        r#type: "number",
                        min: "0",
                        step: "0.5",
                        value: "{fm}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                fm.set(v);
                                fe.set(2.5 * v); // Nyquist criterion with margin
                            }
                        },
                    }
                    span { class: "unit", "Hz" }
                }

                div { class: "form-row",
                    label { r#for: "fe", "Fe =" }
                    input {
                        id: "fe",
                        r#type: "number",
                        min: "0",
                        step: "0.5",
                        value: "{fe}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                fe.set(v);
                            }
                        },
                    }
                    span { class: "unit", "Hz" }
                }

                div { class: "form-row",
                    label { r#for: "n-pts", "N =" }
                    input {
                        id: "n-pts",
                        r#type: "number",
                        min: "4",
                        max: "4096",
                        step: "1",
                        value: "{n_points}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<usize>() {
                                n_points.set(v.clamp(4, 4096));
                            }
                        },
                    }
                    span { class: "unit", "pts" }
                }

                div { class: "form-row",
                    label { "SA =" }
                    input {
                        r#type: "number",
                        readonly: true,
                        value: "{fmt_f(sa)}",
                        tabindex: "-1",
                    }
                    span { class: "unit", "Hz" }
                }

                // ── Legend ─────────────────────────────────────────────────
                div { class: "legend",
                    p { "• Fm : fréquence à mesurer" }
                    p { "• Fe : fréquence d'échantillonnage" }
                    p { "• N  : nombre de points" }
                    p { "• SA : précision spectrale = Fe / N" }
                }
            }

            // ── Right panel: plots ─────────────────────────────────────────
            div { class: "panel-right",
                LineChart {
                    title: "Signal (continu)",
                    x: sig_x,
                    y: sig_y,
                    x_label: "t (s)",
                    y_label: "y",
                    scatter: false,
                    min_y: None,
                }
                LineChart {
                    title: "Échantillons (discrets)",
                    x: samp_x,
                    y: samp_y,
                    x_label: "t (s)",
                    y_label: "y",
                    scatter: true,
                    min_y: None,
                }
                LineChart {
                    title: "Spectre de fréquences – TFD",
                    x: dft_x,
                    y: dft_mag,
                    x_label: "f (Hz)",
                    y_label: "|X[k]|",
                    scatter: false,
                    min_y: Some(0.0),
                }
            }
        }
    }
}

// ─── SinusConfigRow component ─────────────────────────────────────────────────

/// A row of three inputs controlling one sinusoidal component.
///
/// - **ω** (coeff_a): angular frequency; when changed, frequency is derived  
/// - **φ** (coeff_b): phase offset  
/// - **f** (frequency): when changed, coeff_a is derived
#[component]
fn SinusConfigRow(
    index: usize,
    enabled: bool,
    config: SinusConfig,
    on_change: EventHandler<SinusConfig>,
) -> Element {
    let disabled = !enabled;
    let label = format!("Sin {} :", index + 1);

    rsx! {
        div { class: if enabled { "sinus-row" } else { "sinus-row disabled" },
            span { class: "sinus-label", "{label}" }

            // ω (angular frequency)
            div { class: "input-group",
                label { class: "small-label", "ω" }
                input {
                    r#type: "number",
                    disabled: disabled,
                    min: "0",
                    step: "0.1",
                    value: "{config.coeff_a:.4}",
                    oninput: {
                        let cfg = config.clone();
                        move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                on_change.call(SinusConfig::with_coeff_a(cfg.coeff_b, v));
                            }
                        }
                    },
                }
            }

            // φ (phase)
            div { class: "input-group",
                label { class: "small-label", "φ" }
                input {
                    r#type: "number",
                    disabled: disabled,
                    step: "0.1",
                    value: "{config.coeff_b:.4}",
                    oninput: {
                        let cfg = config.clone();
                        move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                let mut new = cfg.clone();
                                new.coeff_b = v;
                                on_change.call(new);
                            }
                        }
                    },
                }
            }

            // f (frequency)
            div { class: "input-group",
                label { class: "small-label", "f (Hz)" }
                input {
                    r#type: "number",
                    disabled: disabled,
                    min: "0",
                    step: "0.1",
                    value: "{config.frequency:.4}",
                    oninput: {
                        let cfg = config.clone();
                        move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                on_change.call(SinusConfig::with_frequency(cfg.coeff_b, v));
                            }
                        }
                    },
                }
            }
        }
    }
}

// ─── LineChart / ScatterChart SVG component ───────────────────────────────────

/// Renders a line chart (or scatter plot) inside an SVG viewport.
///
/// - `scatter = false` → polyline  
/// - `scatter = true`  → crosses for each sample point  
/// - `min_y = Some(v)` → force the y-axis minimum (used for DFT where y ≥ 0)
#[component]
fn LineChart(
    title: &'static str,
    x: Vec<f64>,
    y: Vec<f64>,
    x_label: &'static str,
    y_label: &'static str,
    scatter: bool,
    min_y: Option<f64>,
) -> Element {
    // ── data bounds ────────────────────────────────────────────────────────
    let x_min = x.iter().cloned().fold(f64::INFINITY, f64::min);
    let x_max = x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_min_data = y.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max_data = y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let x_min = if x_min.is_finite() { x_min } else { 0.0 };
    let x_max = if x_max.is_finite() { x_max } else { 1.0 };
    let y_min = min_y.unwrap_or(y_min_data);
    let y_min = if y_min.is_finite() { y_min } else { -1.0 };
    let y_max = if y_max_data.is_finite() { y_max_data } else { 1.0 };

    let x_range = (x_max - x_min).max(1e-10);
    let y_range = (y_max - y_min).max(1e-10);

    // ── polyline points ────────────────────────────────────────────────────
    let pts = if x.is_empty() {
        String::new()
    } else {
        x.iter()
            .zip(y.iter())
            .map(|(&xi, &yi)| {
                let (sx, sy) = to_svg(xi, yi, x_min, x_range, y_min, y_range);
                format!("{sx:.1},{sy:.1}")
            })
            .collect::<Vec<_>>()
            .join(" ")
    };

    // ── y=0 reference line ─────────────────────────────────────────────────
    let zero_y = {
        let (_, sy) = to_svg(0.0, 0.0, x_min, x_range, y_min, y_range);
        sy.clamp(M_T, SVG_H - M_B)
    };
    let show_zero_line = y_min < 0.0 && y_max > 0.0;

    // ── axis ticks ─────────────────────────────────────────────────────────
    let x_ticks = axis_ticks(x_min, x_max, 5);
    let y_ticks = axis_ticks(y_min, y_max, 4);

    let vb = format!("0 0 {SVG_W} {SVG_H}");

    rsx! {
        div { class: "chart-wrapper",
            div { class: "chart-title", "{title}" }
            svg {
                width: "100%",
                height: "100%",
                "viewBox": "{vb}",
                "preserveAspectRatio": "xMidYMid meet",

                // ── background ────────────────────────────────────────────
                rect {
                    x: "{M_L}", y: "{M_T}",
                    width: "{SVG_W - M_L - M_R}",
                    height: "{SVG_H - M_T - M_B}",
                    fill: "#fafafa",
                    stroke: "#ddd",
                    "stroke-width": "1",
                }

                // ── y=0 reference ─────────────────────────────────────────
                if show_zero_line {
                    line {
                        x1: "{M_L}", y1: "{zero_y:.1}",
                        x2: "{SVG_W - M_R}", y2: "{zero_y:.1}",
                        stroke: "#bbb",
                        "stroke-width": "1",
                        "stroke-dasharray": "4,3",
                    }
                }

                // ── x-axis ticks ──────────────────────────────────────────
                for tick_val in x_ticks.iter() {
                    {
                        let (tx, _) = to_svg(*tick_val, 0.0, x_min, x_range, y_min, y_range);
                        let ty_top = SVG_H - M_B;
                        let ty_lbl = SVG_H - M_B + 14.0;
                        let tick_label = fmt_f(*tick_val);
                        rsx! {
                            line {
                                x1: "{tx:.1}", y1: "{ty_top}",
                                x2: "{tx:.1}", y2: "{ty_top + 4.0}",
                                stroke: "#666",
                                "stroke-width": "1",
                            }
                            text {
                                x: "{tx:.1}", y: "{ty_lbl}",
                                "text-anchor": "middle",
                                "font-size": "9",
                                fill: "#555",
                                "{tick_label}"
                            }
                        }
                    }
                }

                // ── y-axis ticks ──────────────────────────────────────────
                for tick_val in y_ticks.iter() {
                    {
                        let (_, ty) = to_svg(0.0, *tick_val, x_min, x_range, y_min, y_range);
                        let tick_label = fmt_f(*tick_val);
                        rsx! {
                            line {
                                x1: "{M_L - 4.0}", y1: "{ty:.1}",
                                x2: "{M_L}", y2: "{ty:.1}",
                                stroke: "#666",
                                "stroke-width": "1",
                            }
                            text {
                                x: "{M_L - 6.0}", y: "{ty + 3.0:.1}",
                                "text-anchor": "end",
                                "font-size": "9",
                                fill: "#555",
                                "{tick_label}"
                            }
                        }
                    }
                }

                // ── axes ──────────────────────────────────────────────────
                // Y axis
                line {
                    x1: "{M_L}", y1: "{M_T}",
                    x2: "{M_L}", y2: "{SVG_H - M_B}",
                    stroke: "#666",
                    "stroke-width": "1",
                }
                // X axis
                line {
                    x1: "{M_L}", y1: "{SVG_H - M_B}",
                    x2: "{SVG_W - M_R}", y2: "{SVG_H - M_B}",
                    stroke: "#666",
                    "stroke-width": "1",
                }

                // ── axis labels ───────────────────────────────────────────
                text {
                    x: "{SVG_W / 2.0}",
                    y: "{SVG_H - 2.0}",
                    "text-anchor": "middle",
                    "font-size": "10",
                    fill: "#444",
                    "{x_label}"
                }
                text {
                    x: "12",
                    y: "{(M_T + SVG_H - M_B) / 2.0}",
                    "text-anchor": "middle",
                    "font-size": "10",
                    fill: "#444",
                    transform: "rotate(-90, 12, {(M_T + SVG_H - M_B) / 2.0})",
                    "{y_label}"
                }

                // ── data ─────────────────────────────────────────────────
                if scatter {
                    // Draw ✕ crosses for discrete sample points
                    for (xi, yi) in x.iter().zip(y.iter()) {
                        {
                            let (sx, sy) = to_svg(*xi, *yi, x_min, x_range, y_min, y_range);
                            let d = 4.0_f64;
                            rsx! {
                                line {
                                    x1: "{sx - d:.1}", y1: "{sy - d:.1}",
                                    x2: "{sx + d:.1}", y2: "{sy + d:.1}",
                                    stroke: "#cc2200",
                                    "stroke-width": "1.5",
                                }
                                line {
                                    x1: "{sx + d:.1}", y1: "{sy - d:.1}",
                                    x2: "{sx - d:.1}", y2: "{sy + d:.1}",
                                    stroke: "#cc2200",
                                    "stroke-width": "1.5",
                                }
                            }
                        }
                    }
                } else {
                    // Continuous line
                    polyline {
                        points: "{pts}",
                        stroke: "#0066cc",
                        "stroke-width": "1.5",
                        fill: "none",
                    }
                }
            }
        }
    }
}

// ─── Embedded CSS ─────────────────────────────────────────────────────────────

const CSS: &str = r#"
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

body {
    font-family: system-ui, "Segoe UI", Helvetica, Arial, sans-serif;
    font-size: 13px;
    background: #f0f2f5;
    color: #222;
}

.app {
    display: flex;
    height: 100vh;
    overflow: hidden;
}

/* ── Left panel ──────────────────────────────────── */
.panel-left {
    width: 300px;
    min-width: 260px;
    background: #fff;
    border-right: 1px solid #d0d7de;
    padding: 12px 14px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.panel-title {
    font-size: 15px;
    font-weight: 600;
    color: #0a5a9c;
    margin-bottom: 8px;
    padding-bottom: 6px;
    border-bottom: 2px solid #0a5a9c;
}

.section-title {
    font-weight: 600;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: .05em;
    color: #555;
    margin-top: 10px;
    margin-bottom: 4px;
}

/* ── Form rows ───────────────────────────────────── */
.form-row {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 4px;
}

.form-row label {
    width: 42px;
    text-align: right;
    flex-shrink: 0;
    font-size: 12px;
}

.form-row input, .form-row select {
    flex: 1;
    padding: 3px 5px;
    border: 1px solid #c8ccd0;
    border-radius: 4px;
    font-size: 12px;
    min-width: 0;
}

.form-row input:focus, .form-row select:focus {
    outline: none;
    border-color: #0a5a9c;
    box-shadow: 0 0 0 2px rgba(10,90,156,.15);
}

.form-row input[readonly] {
    background: #f4f5f6;
    color: #555;
}

.unit {
    font-size: 11px;
    color: #777;
    width: 26px;
    flex-shrink: 0;
}

/* ── Sinusoïde rows ──────────────────────────────── */
.sinus-row {
    background: #f6f8fa;
    border: 1px solid #d0d7de;
    border-radius: 6px;
    padding: 6px 8px;
    margin-bottom: 4px;
    transition: opacity .2s;
}

.sinus-row.disabled {
    opacity: 0.38;
    pointer-events: none;
}

.sinus-label {
    display: block;
    font-weight: 600;
    font-size: 11px;
    color: #0a5a9c;
    margin-bottom: 4px;
}

.input-group {
    display: flex;
    align-items: center;
    gap: 5px;
    margin-bottom: 3px;
}

.small-label {
    width: 36px;
    font-size: 11px;
    color: #555;
    text-align: right;
    flex-shrink: 0;
}

.input-group input {
    flex: 1;
    padding: 2px 4px;
    border: 1px solid #c8ccd0;
    border-radius: 4px;
    font-size: 12px;
    min-width: 0;
}

.input-group input:focus {
    outline: none;
    border-color: #0a5a9c;
}

/* ── Legend ─────────────────────────────────────── */
.legend {
    margin-top: 12px;
    padding: 8px 10px;
    background: #f0f4f8;
    border-radius: 6px;
    font-size: 11px;
    color: #555;
    line-height: 1.7;
}

/* ── Right panel ─────────────────────────────────── */
.panel-right {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 12px;
    overflow-y: auto;
    min-width: 0;
}

/* ── Chart wrapper ───────────────────────────────── */
.chart-wrapper {
    flex: 1;
    min-height: 140px;
    background: #fff;
    border: 1px solid #d0d7de;
    border-radius: 6px;
    padding: 6px 8px 4px;
    display: flex;
    flex-direction: column;
}

.chart-title {
    font-size: 11px;
    font-weight: 600;
    color: #444;
    text-align: center;
    margin-bottom: 2px;
    letter-spacing: .03em;
}

.chart-wrapper svg {
    flex: 1;
    display: block;
}
"#;
