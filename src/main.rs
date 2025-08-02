mod initial_state_model;
mod satellite_state;

fn main() {
    if let Err(e) = satellite_state::demo_deorbit() {
        eprintln!("Error: {}", e);
    }
}
