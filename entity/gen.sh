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

sed "s/: Vec<u8>,/: uuid::Uuid,/" -i "$SRC/"*.rs
sed "s/pub content: String,/pub content: super::custom::flash_card::FlashCardContent,/" -i "$SRC/flash_card.rs"