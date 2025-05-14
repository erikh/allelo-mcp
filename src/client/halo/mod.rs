use std::sync::LazyLock;

#[derive(Debug, Clone, Default)]
pub(crate) struct Client {}

#[derive(Debug, Clone, Default)]
pub(crate) struct Fault {
    summary: String,
}

impl Fault {
    pub(crate) fn summary(&self) -> String {
        self.summary.clone()
    }
}

#[derive(Debug, Clone, Default, thiserror::Error)]
pub enum Error {
    #[default]
    #[error("unimplemented")]
    Unimplemented,
}

impl Into<rmcp::model::ErrorData> for Error {
    fn into(self) -> rmcp::Error {
        rmcp::model::ErrorData::new(
            rmcp::model::ErrorCode::INTERNAL_ERROR,
            self.to_string(),
            None,
        )
    }
}

static MOCK: LazyLock<Vec<Fault>> = LazyLock::new(|| {
    vec![
        Fault {
            summary: String::from("this is an example summary"),
        },
        Fault {
            summary: String::from("this is another example summary"),
        },
    ]
});

impl Client {
    pub(crate) fn list_faults(&self) -> Result<Vec<Fault>, Error> {
        Ok(MOCK.clone())
    }
}
