// use crate::bindings;
// use log::{Log, Metadata, Record};
// use std::fmt::Debug;
// use std::os::raw::c_char;
//
// pub unsafe extern "C" fn log_session(
//     msg: *const c_char,
//     len: usize,
//     cat: *const c_char,
//     cat_len: usize,
//     level: bindings::LOG_LEVEL,
// ) {
//     let target = std::slice::from_raw_parts(cat as *const u8, cat_len);
//     let Ok(target) = std::str::from_utf8(target) else {
//         return;
//     };
//
//     let log = log::logger();
//     let level = session_log_level_to_log_level(level);
//
//     let metadata = Metadata::builder().target(target).level(level).build();
//
//     if log.enabled(&metadata) {
//         let msg = std::slice::from_raw_parts(msg as *const u8, len);
//         let Ok(msg) = std::str::from_utf8(msg) else {
//             return;
//         };
//
//         log.log(
//             &Record::builder()
//                 .metadata(metadata)
//                 .args(format_args!("{msg}"))
//                 .build(),
//         )
//     }
// }

// pub fn session_log_level_to_log_level(level: bindings::LOG_LEVEL) -> log::Level {
//     match level {
//         bindings::LOG_LEVEL_LOG_LEVEL_TRACE => log::Level::Trace,
//         bindings::LOG_LEVEL_LOG_LEVEL_INFO => log::Level::Info,
//         bindings::LOG_LEVEL_LOG_LEVEL_WARN => log::Level::Warn,
//         bindings::LOG_LEVEL_LOG_LEVEL_ERROR | bindings::LOG_LEVEL_LOG_LEVEL_CRITICAL => {
//             log::Level::Error
//         }
//         _ => log::Level::Debug,
//     }
// }

// pub fn log_level_to_session_log_level(level: log::Level) -> bindings::LOG_LEVEL {
//     match level {
//         log::Level::Trace => bindings::LOG_LEVEL_LOG_LEVEL_TRACE,
//         log::Level::Info => bindings::LOG_LEVEL_LOG_LEVEL_INFO,
//         log::Level::Warn => bindings::LOG_LEVEL_LOG_LEVEL_WARN,
//         log::Level::Error => bindings::LOG_LEVEL_LOG_LEVEL_ERROR,
//         log::Level::Debug => bindings::LOG_LEVEL_LOG_LEVEL_DEBUG,
//     }
// }
