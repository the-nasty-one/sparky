use leptos::prelude::*;

/// SVG circular gauge component.
///
/// Renders a 240-degree arc that fills based on `value` (0-100).
/// Uses stroke-dasharray/stroke-dashoffset technique.
/// Color transitions from green -> yellow -> red based on thresholds.
#[component]
pub fn Gauge(
    /// Value from 0.0 to 100.0
    value: f32,
    /// Label text below the gauge
    label: String,
    /// Unit string displayed after value (e.g., "%", "°C")
    unit: String,
    /// Stroke color for the filled arc
    color: String,
) -> impl IntoView {
    let SIZE: f32 = 120.0;
    let STROKE_WIDTH: f32 = 8.0;
    let RADIUS: f32 = (SIZE - STROKE_WIDTH) / 2.0;
    let CENTER: f32 = SIZE / 2.0;

    // 240 degrees of the circle (2/3)
    let ARC_DEGREES: f32 = 240.0;
    let circumference = 2.0 * std::f32::consts::PI * RADIUS;
    let arcLength = circumference * (ARC_DEGREES / 360.0);

    // clamp value to 0-100
    let clampedValue = value.clamp(0.0, 100.0);
    let filledLength = arcLength * (clampedValue / 100.0);

    // The gap portion of the dasharray (non-arc part)
    let gapLength = circumference - arcLength;

    // Background arc dasharray: show the arc portion, hide the rest
    let bgDasharray = format!("{arcLength} {gapLength}");

    // Filled arc: draw exactly filledLength of stroke, then hide everything else.
    // Using circumference as the gap ensures the unfilled portion is fully hidden.
    let fillDasharray = format!("{filledLength} {circumference}");

    // Rotate so the arc starts at bottom-left (210 degrees from 3 o'clock)
    // The arc spans from 210° to 330° going clockwise through top
    // SVG circle starts at 3 o'clock. We rotate -90 for top, then +30 more = 150° total
    let ROTATION: f32 = 150.0;

    let displayValue = if value == value.floor() {
        format!("{:.0}", clampedValue)
    } else {
        format!("{:.1}", clampedValue)
    };

    view! {
        <div class="gauge-container">
            <svg
                width=format!("{SIZE}")
                height=format!("{SIZE}")
                viewBox=format!("0 0 {SIZE} {SIZE}")
                class="gauge-svg"
                style=format!("transform: rotate({ROTATION}deg)")
            >
                // Background arc
                <circle
                    cx=format!("{CENTER}")
                    cy=format!("{CENTER}")
                    r=format!("{RADIUS}")
                    class="gauge-bg"
                    stroke-width=format!("{STROKE_WIDTH}")
                    stroke-dasharray=bgDasharray.clone()
                    stroke-dashoffset="0"
                />
                // Filled arc
                <circle
                    cx=format!("{CENTER}")
                    cy=format!("{CENTER}")
                    r=format!("{RADIUS}")
                    class="gauge-fill"
                    stroke=color.clone()
                    stroke-width=format!("{STROKE_WIDTH}")
                    stroke-dasharray=fillDasharray
                    stroke-dashoffset="0"
                />
                // Center text (counter-rotate so text is upright)
                <text
                    x=format!("{CENTER}")
                    y=format!("{}", CENTER - 6.0)
                    class="gauge-text gauge-value"
                    transform=format!("rotate({} {} {})", -ROTATION, CENTER, CENTER)
                >
                    {displayValue}
                </text>
                <text
                    x=format!("{CENTER}")
                    y=format!("{}", CENTER + 12.0)
                    class="gauge-text gauge-unit"
                    transform=format!("rotate({} {} {})", -ROTATION, CENTER, CENTER)
                >
                    {unit}
                </text>
            </svg>
            <span class="gauge-label">{label}</span>
        </div>
    }
}
