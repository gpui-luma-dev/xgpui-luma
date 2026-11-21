//! Built-in keywords and functions. Look functions come from the token source instead.

use crate::color::{Color, RgbSpace};
use crate::value::{Easing, Kind, Motion, ShadowLayer, TextStyle, Value};

/// Core properties: name, kind, and whether parts inherit it from `root`.
pub(crate) const CORE_PROPERTIES: &[(&str, Kind, bool)] = &[
    ("background", Kind::Color, false),
    ("foreground", Kind::Color, true),
    ("border", Kind::Color, false),
    ("focus_border", Kind::Color, false),
    ("border_width", Kind::Length, false),
    ("radius", Kind::Length, false),
    ("padding", Kind::Length, false),
    ("padding_x", Kind::Length, false),
    ("padding_y", Kind::Length, false),
    ("margin", Kind::Length, false),
    ("gap", Kind::Length, false),
    ("width", Kind::Length, false),
    ("height", Kind::Length, false),
    ("min_width", Kind::Length, false),
    ("max_width", Kind::Length, false),
    ("min_height", Kind::Length, false),
    ("max_height", Kind::Length, false),
    ("opacity", Kind::Number, false),
    ("shadow", Kind::Shadow, false),
    ("font_family", Kind::Font, true),
    ("font_size", Kind::Length, true),
    ("line_height", Kind::Length, true),
    ("font_weight", Kind::Weight, true),
    ("transition", Kind::Motion, false),
];

pub(crate) fn core_property(name: &str) -> Option<(Kind, bool)> {
    CORE_PROPERTIES.iter().find(|(n, _, _)| *n == name).map(|(_, k, i)| (*k, *i))
}

/// A keyword's value (`none`, `transparent`, `black`, `white`, `instant`), or `None` if `name` is not one.
pub(crate) fn keyword(name: &str) -> Option<Value> {
    Some(match name {
        "none" => Value::None,
        "transparent" => Value::Color(Color::TRANSPARENT),
        "black" => Value::Color(Color::BLACK),
        "white" => Value::Color(Color::WHITE),
        "instant" => Value::Motion(Motion::Instant),
        _ => return None,
    })
}

/// Built-in function: (min args, max args, return kind). `first` and `exact` are handled by the evaluator.
pub(crate) fn builtin(name: &str) -> Option<(usize, usize, Option<Kind>)> {
    Some(match name {
        "first" => (1, usize::MAX, None),
        "exact" => (1, 1, None),
        "min" | "max" => (1, usize::MAX, Some(Kind::Number)),
        "alpha" => (2, 2, Some(Kind::Color)),
        "mix" => (3, 3, Some(Kind::Color)),
        "gray" => (1, 2, Some(Kind::Color)),
        "srgb" => (3, 4, Some(Kind::Color)),
        "hsla" => (3, 4, Some(Kind::Color)),
        "oklch" => (3, 4, Some(Kind::Color)),
        "oklab" => (3, 4, Some(Kind::Color)),
        "color" => (4, 5, Some(Kind::Color)),
        "shadow" => (5, 5, Some(Kind::Shadow)),
        "box_shadow" | "inset_shadow" => (5, 5, Some(Kind::Shadow)),
        "text" => (3, 3, Some(Kind::Text)),
        "timed" => (2, 2, Some(Kind::Motion)),
        "spring" => (2, 2, Some(Kind::Motion)),
        _ => return None,
    })
}

fn num(v: &Value, what: &str) -> Result<f32, String> {
    v.as_number().ok_or_else(|| format!("{what} must be a number, found {}", v.describe()))
}

fn color(v: &Value, what: &str) -> Result<Color, String> {
    v.as_color().ok_or_else(|| format!("{what} must be a color, found {}", v.describe()))
}

fn opt_alpha(args: &[Value], i: usize) -> Result<f32, String> {
    args.get(i).map_or(Ok(1.0), |v| num(v, "alpha"))
}

fn layer(color: Color, args: &[Value], inset: bool) -> Result<Value, String> {
    Ok(Value::Shadow(
        vec![ShadowLayer {
            x: num(&args[1], "x")?,
            y: num(&args[2], "y")?,
            blur: num(&args[3], "blur")?,
            spread: num(&args[4], "spread")?,
            color,
            inset,
        }]
        .into(),
    ))
}

/// Call a built-in other than `first` and `exact` with evaluated arguments.
pub(crate) fn call(name: &str, args: &[Value]) -> Result<Value, String> {
    Ok(match name {
        "min" | "max" => {
            let mut values = args.iter().map(|a| num(a, "argument"));
            let first = values.next().unwrap_or(Ok(0.0))?;
            let mut acc = first;
            for v in values {
                let v = v?;
                acc = if name == "min" { acc.min(v) } else { acc.max(v) };
            }
            Value::Number(acc)
        }
        "alpha" => Value::Color(color(&args[0], "the first argument")?.with_alpha(num(&args[1], "alpha")?)),
        "mix" => Value::Color(
            color(&args[0], "the first argument")?
                .mix(color(&args[1], "the second argument")?, num(&args[2], "the amount")?),
        ),
        "gray" => {
            let l = num(&args[0], "lightness")? / 100.0;
            Value::Color(Color::srgb(l, l, l, opt_alpha(args, 1)?))
        }
        "srgb" => Value::Color(Color::srgb(
            num(&args[0], "r")?,
            num(&args[1], "g")?,
            num(&args[2], "b")?,
            opt_alpha(args, 3)?,
        )),
        "hsla" => Value::Color(Color::hsla(
            num(&args[0], "hue")?,
            num(&args[1], "saturation")?,
            num(&args[2], "lightness")?,
            opt_alpha(args, 3)?,
        )),
        "oklch" => Value::Color(Color::oklch(
            num(&args[0], "l")?,
            num(&args[1], "c")?,
            num(&args[2], "h")?,
            opt_alpha(args, 3)?,
        )),
        "oklab" => Value::Color(Color::oklab(
            num(&args[0], "l")?,
            num(&args[1], "a")?,
            num(&args[2], "b")?,
            opt_alpha(args, 3)?,
        )),
        "color" => {
            let space = args[0].as_str().and_then(RgbSpace::parse).ok_or_else(|| {
                "color() needs a space: srgb, srgb_linear, display_p3, a98_rgb, prophoto_rgb or rec2020".to_string()
            })?;
            Value::Color(Color::rgb(
                space,
                num(&args[1], "r")?,
                num(&args[2], "g")?,
                num(&args[3], "b")?,
                opt_alpha(args, 4)?,
            ))
        }
        "shadow" => layer(Color::BLACK.with_alpha(num(&args[0], "alpha")?), args, false)?,
        "box_shadow" => layer(color(&args[0], "the shadow color")?, args, false)?,
        "inset_shadow" => layer(color(&args[0], "the shadow color")?, args, true)?,
        "text" => Value::Text(TextStyle {
            size: num(&args[0], "size")?,
            line_height: num(&args[1], "line height")?,
            weight: num(&args[2], "weight")?,
        }),
        "timed" => {
            let easing = args[1].as_str().and_then(Easing::parse).ok_or_else(|| {
                "timed() needs an easing: linear, in_quad, out_quad, in_out_quad, in_cubic, out_cubic or in_out_cubic"
                    .to_string()
            })?;
            Value::Motion(Motion::Timed { duration_ms: num(&args[0], "duration")?, easing })
        }
        "spring" => {
            Value::Motion(Motion::Spring { stiffness: num(&args[0], "stiffness")?, damping: num(&args[1], "damping")? })
        }
        other => return Err(format!("unknown function \"{other}\"")),
    })
}
