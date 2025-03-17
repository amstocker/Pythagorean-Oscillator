mod tracker;
mod voice;

pub use tracker::CycleTracker;


pub struct Engine {
    cycle_tracker: CycleTracker<8192>
}