pub(crate) use anyhow::Result as AnyHowResult;
pub(crate) type CustomAppResult<T> = Result<T, String>;

pub(crate) trait MapAnyHowResultToMessage<T> {
    fn map_to_message(self) -> CustomAppResult<T>;
}

impl<T> MapAnyHowResultToMessage<T> for AnyHowResult<T> {
    fn map_to_message(self) -> CustomAppResult<T> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(e.to_string().trim_start_matches("Error: ").to_string()),
        }
    }
}
