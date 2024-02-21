#!/bin/sh

SCRIPT=$(dirname "$0")
PROJECT=$(cd $SCRIPT && cargo locate-project  --message-format=plain)
DIR=$(dirname $PROJECT)
SRC="$DIR/src"

sea-orm-cli generate entity													\
  -u mysql://root:qwerty@localhost:3306/flashmind							\
  -o "$SRC/" --lib															\
  --with-serde both --serde-skip-deserializing-primary-key					\
  --with-copy-enums 														\
  --enum-extra-derives "ts_rs::TS" --enum-extra-attributes "ts(export)"		\
  --model-extra-derives "ts_rs::TS" --model-extra-attributes "ts(export)"

echo "pub mod custom;" >> "$SRC/lib.rs"

for f in "$SRC"/*.rs; do
	name=$(basename -s .rs $f)
	# shellcheck disable=SC3059
  	name=${name^}

	sed "/pub struct Model {/ i #[ts(rename = \"$name\")]" -i "$f"
done

sed -e "s/: Vec<u8>,/: uuid::Uuid,/" \
	-e "s/pub creator: u32,/#[serde(skip_deserializing)]\n\tpub creator: u32,/" \
	-e "s/Flash_card/FlashCard/" \
	-i "$SRC/"*.rs
sed "s/pub content: String,/pub content: super::custom::flash_card::FlashCardContent,/" -i "$SRC/flash_card.rs"

sed "s/#\[ts(export)\]//" -i "$SRC/deck_cards.rs" "$SRC/followed_decks.rs"

cargo +nightly fmt