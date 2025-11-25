/// Core functionality modules
///
/// Contains the main business logic for command recording,
/// retrieval, searching, and project detection.

pub mod project_detector;
pub mod recorder;
pub mod retriever;
pub mod searcher;

pub use project_detector::ProjectDetector;
pub use recorder::Recorder;
pub use retriever::Retriever;
pub use searcher::Searcher;
