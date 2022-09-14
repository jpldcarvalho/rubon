
#[allow(clippy::derive_partialeq_without_eq)]
mod overlay_proto {
    include!(concat!(env!("OUT_DIR"), "/overlay.pb.rs"));
}

pub mod overlay;
