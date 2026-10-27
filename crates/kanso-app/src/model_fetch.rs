//! Models Kanso downloads once and runs on this device: related notes'
//! embedding model (`related.rs`) and voice input's speech model
//! (`voice.rs`). The download, its checks and its resuming are the
//! engine's (`kanso_engine::discovery::model_fetch`), which the phones
//! fetch the embedding model with too.

pub use kanso_engine::discovery::model_fetch::*;
