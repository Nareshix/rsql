use sqlformat::{FormatOptions, Indent, QueryParams, format};

/// This nicely formats the sql string.
/// 
/// Useful for vscode hover over fn
pub(crate) fn format_sql(sql: &str) -> String {
    let options = FormatOptions {
        indent: Indent::Tabs,
        ..Default::default()
    };
    format(sql, &QueryParams::None, &options)
}