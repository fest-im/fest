// From http://gtk-rs.org/tuto/closures
//
// This takes a comma separated list of elements which are then cloned for use
// in a move closure. A `=>` separates the element(s) from the closure.
//
// This allows moved elements to be reused across multiple connected callbacks
// without extra boilerplate code (in most cases).
#[macro_export]
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

// ===================== Logging =====================
extern crate chrono;
extern crate fern;

/// Sets up the logging front end.
pub(crate) fn set_up_logging() {
    use crate::log;
    set_up_fern_dispatch()
        // set the default log level
        .level(log::LevelFilter::Warn)
        // set module (actually, it's target) specific log levels
        .level_for("fest", log::LevelFilter::Trace)
        // output to stdout
        .chain(::std::io::stdout())
        .apply().unwrap();

    debug!("finished setting up logging! yay!");
    trace!("*tap* *tap* is this thing on? test test");

}


/// creates a `fern::Dispatch` object. if the 'logging_color' feature is enabled, it outputs
/// ansi escape sequencen to color the logs.
#[cfg(feature = "logging_color")]
fn set_up_fern_dispatch() -> fern::Dispatch {
    use util::fern::colors::{Color, ColoredLevelConfig};
    // configure colors for the whole line
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        // we actually don't need to specify the color for debug and info, they are white by default
        .info(Color::White)
        .debug(Color::White)
        // depending on the terminals color scheme, this is the same as the background color
        .trace(Color::BrightBlack);
    // configure colors for the name of the level.
    // since almost all of them are the some as the color for the whole line, we just clone
    // `colors_line` and overwrite our changes
    let colors_level = colors_line.clone()
        .info(Color::Green);
    // here we set up our fern Dispatch
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}[{date}][{target}][{level}{color_line}] {message}\x1B[0m",
                color_line = format_args!("\x1B[{}m", colors_line.get_color(&record.level()).to_fg_str()),
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ));
        })
}

/// creates a `fern::Dispatch` object. if the 'logging_color' feature is enabled, it outputs
/// ansi escape sequencen to color the logs.
#[cfg(not(feature = "logging_color"))]
fn set_up_fern_dispatch() -> fern::Dispatch {
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{date}][{target}][{level}] {message}",
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                target = record.target(),
                level = record.level(),
                message = message,
            ));
        })
}
