#[cfg(target_arch = "wasm32")]
pub type Instant = web_time::Instant;
#[cfg(target_arch = "wasm32")]
pub type Duration = web_time::Duration;

#[cfg(not(target_arch = "wasm32"))]
pub type Instant = std::time::Instant;
#[cfg(not(target_arch = "wasm32"))]
pub type Duration = std::time::Duration;
