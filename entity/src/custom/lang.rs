use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum Language {
	Pl,
	// Polish
	En,
	// English
	EnUk,
	// British English
	EnUs, // American English
}
