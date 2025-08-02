mod initial_state_model;
mod satellite_state;

fn main() {
    if let Err(e) = satkit::utils::update_datafiles(None, false) {
        eprintln!("Error downloading data files: {}", e);
    }

    if let Err(e) = satellite_state::demo_deorbit() {
        eprintln!("Error: {}", e);
    }
}
