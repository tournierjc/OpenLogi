//! Clear the manual app-profile override on a device (return to Default).
//!
//! ```sh
//! cargo run -p openlogi-ipc --example reset_app_profile -- unit:31384705
//! ```

use tarpc::context;

#[tokio::main]
async fn main() {
    let device_key = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "unit:31384705".to_string());
    let conn = openlogi_ipc::client::connect()
        .await
        .expect("connect to agent");
    let ok = conn
        .client
        .set_app_profile_override(context::current(), device_key.clone(), None)
        .await
        .expect("RPC");
    if ok {
        eprintln!("profile override cleared for {device_key}");
    } else {
        eprintln!("agent rejected profile reset for {device_key}");
        std::process::exit(1);
    }
}
