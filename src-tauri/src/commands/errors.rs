use libdav::sd::BootstrapError;
use specta::Type;

#[derive(Debug, thiserror::Error)]
pub(crate) enum CommandError {
    #[error("Diesel error {0:?}")]
    Diesel(#[from] diesel::result::Error),
    #[error("Could not connect to caldav")]
    CaldavBootstrap(#[from] BootstrapError),
    #[error("Tauri error {0:?}")]
    Tauri(#[from] tauri::Error),
    #[error("Error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl From<CommandError> for String {
    fn from(value: CommandError) -> Self {
        value.to_string()
    }
}

impl Type for CommandError {
    fn inline(
        type_map: &mut specta::TypeCollection,
        generics: specta::Generics,
    ) -> specta::datatype::DataType {
        String::inline(type_map, generics)
    }
}

impl serde::Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
