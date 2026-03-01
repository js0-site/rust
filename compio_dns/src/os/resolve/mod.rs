mod parse;

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        mod win;
        pub(crate) use win::load_conf;
    } else {
        mod unix;
        pub(crate) use unix::load_conf;
    }
}
