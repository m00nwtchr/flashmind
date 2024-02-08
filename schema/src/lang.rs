use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Language {
	Pl,   // Polish
	En, // English
	EnUk, // British English
	EnUs, // American English
}
