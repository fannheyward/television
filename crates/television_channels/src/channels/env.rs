use devicons::FileIcon;

use super::OnAir;
use crate::entry::{Entry, PreviewType};
use television_fuzzy::matcher::{config::Config, Matcher};
use television_utils::utils::indices::sep_name_and_value_indices;
use television_utils::utils::strings::preprocess_line;

#[derive(Debug, Clone)]
struct EnvVar {
    name: String,
    value: String,
}

#[allow(clippy::module_name_repetitions)]
pub struct Channel {
    matcher: Matcher<EnvVar>,
    file_icon: FileIcon,
}

const NUM_THREADS: usize = 1;
const FILE_ICON_STR: &str = "config";

impl Channel {
    pub fn new() -> Self {
        let matcher = Matcher::new(Config::default().n_threads(NUM_THREADS));
        let injector = matcher.injector();
        for (name, value) in std::env::vars() {
            let () = injector.push(
                EnvVar {
                    name: preprocess_line(&name),
                    value: preprocess_line(&value),
                },
                |e, cols| {
                    cols[0] = (e.name.clone() + &e.value).into();
                },
            );
        }
        Channel {
            matcher,
            file_icon: FileIcon::from(FILE_ICON_STR),
        }
    }
}

impl Default for Channel {
    fn default() -> Self {
        Self::new()
    }
}

impl OnAir for Channel {
    fn find(&mut self, pattern: &str) {
        self.matcher.find(pattern);
    }

    fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry> {
        self.matcher.tick();
        self.matcher
            .results(num_entries, offset)
            .into_iter()
            .map(|item| {
                let (
                    name_indices,
                    value_indices,
                    should_add_name_indices,
                    should_add_value_indices,
                ) = sep_name_and_value_indices(
                    &mut item.match_indices.iter().map(|i| i.0).collect(),
                    u32::try_from(item.inner.name.len()).unwrap(),
                );

                let mut entry =
                    Entry::new(item.inner.name.clone(), PreviewType::EnvVar)
                        .with_value(item.inner.value.clone())
                        .with_icon(self.file_icon);

                if should_add_name_indices {
                    entry = entry.with_name_match_ranges(
                        name_indices.into_iter().map(|i| (i, i + 1)).collect(),
                    );
                }

                if should_add_value_indices {
                    entry = entry.with_value_match_ranges(
                        value_indices
                            .into_iter()
                            .map(|i| (i, i + 1))
                            .collect(),
                    );
                }

                entry
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            Entry::new(item.inner.name.clone(), PreviewType::EnvVar)
                .with_value(item.inner.value.clone())
                .with_icon(self.file_icon)
        })
    }

    fn result_count(&self) -> u32 {
        self.matcher.matched_item_count
    }

    fn total_count(&self) -> u32 {
        self.matcher.total_item_count
    }

    fn running(&self) -> bool {
        self.matcher.status.running
    }

    fn shutdown(&self) {}
}