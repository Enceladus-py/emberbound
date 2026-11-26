//! Storage adapter for the best run: the browser's `localStorage` on the
//! web, a small text file in the working directory natively. Storage is a
//! nicety, so every failure (private mode, quota, IO, corrupt data) is
//! silently ignored and simply means "no record".

use bevy::prelude::*;
use emberbound_core::BestRunRecord;
use emberbound_domain::records::BestRun;

pub struct RecordsPlugin;

impl Plugin for RecordsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_best_run)
            .add_systems(Update, save_best_run.run_if(resource_changed::<BestRunRecord>));
    }
}

fn load_best_run(mut record: ResMut<BestRunRecord>) {
    record.best = storage::read().as_deref().and_then(BestRun::from_text);
}

/// Writes the record down whenever a run breaks it.
fn save_best_run(record: Res<BestRunRecord>) {
    if let (true, Some(best)) = (record.new_best, record.best) {
        storage::write(&best.to_text());
    }
}

#[cfg(target_arch = "wasm32")]
mod storage {
    const KEY: &str = "emberbound.best_run";

    fn local_storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    pub fn read() -> Option<String> {
        local_storage()?.get_item(KEY).ok().flatten()
    }

    pub fn write(text: &str) {
        if let Some(storage) = local_storage() {
            let _ = storage.set_item(KEY, text);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod storage {
    const PATH: &str = "emberbound-best-run.txt";

    pub fn read() -> Option<String> {
        std::fs::read_to_string(PATH).ok()
    }

    pub fn write(text: &str) {
        let _ = std::fs::write(PATH, text);
    }
}
