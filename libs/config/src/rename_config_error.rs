use std::rc::Rc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenameConfigError {
	#[error("multiple entries in old config map to same key after rename")]
	KeyClash(Rc<str>),
}
