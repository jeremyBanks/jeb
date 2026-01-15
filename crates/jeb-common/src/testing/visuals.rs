use {
    itertools::{
        EitherOrBoth::{
            Both,
            Left,
            Right,
        },
        Itertools,
    },
    std::collections::HashMap,
};

pub fn concat(left: impl AsRef<str>, separator: impl AsRef<str>, right: impl AsRef<str>) -> String {
    // joins two strings line-by-line, padding out all left lines with spaces to
    // be the same as their maximum length (unless both sides are empty)
    let left_lines = left.as_ref().lines().collect::<Vec<_>>();
    let right_lines = right.as_ref().lines().collect::<Vec<_>>();
    let separator = separator.as_ref();
    let max_left_line_length = left_lines.iter().map(|line| line.len()).max().unwrap_or(0);
    let mut lines = vec![];
    for pair in left_lines.iter().zip_longest(right_lines.iter()) {
        let (left_line, right_line) = match pair {
            Both(l, r) => (*l, *r),
            Left(l) => (*l, ""),
            Right(r) => ("", *r),
        };
        if left_line.is_empty() && right_line.is_empty() {
            lines.push("".to_string());
        } else {
            lines.push(format!(
                "{left_line:max_left_line_length$}{separator}{right_line}",
            ));
        }
    }
    lines.join("\n")
}

pub fn render_d1d<T: Into<i128>>(points: impl IntoIterator<Item = T>) -> String {
    let points = points.into_iter().map(Into::into).collect::<Vec<_>>();
    concat(render_d1(points.clone()), "  |  ", render_1d(points))
}

pub fn render_d2d<T: Into<i128>, U: Into<i128>>(
    points: impl IntoIterator<Item = (T, U)>,
) -> String {
    let points = points
        .into_iter()
        .map(|(x, y)| (x.into(), y.into()))
        .collect::<Vec<_>>();
    concat(render_d2(points.clone()), "  |  ", render_2d(points))
}

pub fn render_1d<T: Into<i128>>(points: impl IntoIterator<Item = T>) -> String {
    render_2d(points.into_iter().enumerate().map(|(i, x)| (x, i as i128)))
}

pub fn render_d1<T: Into<i128>>(points: impl IntoIterator<Item = T>) -> String {
    render_2d(points.into_iter().enumerate().map(|(i, x)| (i as i128, x)))
}

pub fn render_d2<T: Into<i128>, U: Into<i128>>(points: impl IntoIterator<Item = (T, U)>) -> String {
    render_2d(points.into_iter().map(|(x, y)| (y, x)))
}

pub fn render_2d<T: Into<i128>, U: Into<i128>>(points: impl IntoIterator<Item = (T, U)>) -> String {
    let spread = 9;
    let gradient = b".:i123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ%!$&#@";

    let mut seen = HashMap::new();
    let mut min_x = i128::MAX;
    let mut max_x = i128::MIN;
    let mut min_y = i128::MAX;
    let mut max_y = i128::MIN;
    for (i, (x, y)) in points.into_iter().enumerate() {
        let x = x.into();
        let y = y.into();
        seen.insert((x, y), i);
        if x < min_x {
            min_x = x;
        }
        if x > max_x {
            max_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if y > max_y {
            max_y = y;
        }
    }
    let mut lines = vec![];
    for y in min_y..=max_y {
        let mut line = String::new();
        for x in min_x..=max_x {
            if let Some(index) = seen.get(&(x, y)) {
                line.push(gradient[(index / spread) % gradient.len()] as char);
            } else {
                line.push(' ');
            }
        }
        lines.push(line);
    }
    let rendered = format!("\n{}\n", lines.join("\n"));

    rendered
}
