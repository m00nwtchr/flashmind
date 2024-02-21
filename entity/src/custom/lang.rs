use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Language {
	Pl,   // Polish
	En,   // English
	EnUk, // British English
	EnUs, // American English
}
