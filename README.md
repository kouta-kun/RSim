# RSim
Game for GBAJam 2024 about fishing in a river.

This uses a [fork of agb.rs](https://github.com/kouta-kun/agb/) that opens up certain APIs to simplify code. TODO try to make it run on upstream agb.rs

## Status
As of 15 August 2024, I completely forgot about this game and remembered I was working on it like 2 days before the game jam ended, so I don't expect feature completeness before it ends on August 18th.

Implemented:

- River generation
- Tree cutting
- Tree regrowth after some time
- Using wood planks to build bridges

TODO:

- Craft fishing tools (fishing rod, nets, etc.) using wood planks
- Food inventory (fish parts, gathered fruits)
- Hunger system
- Point system


## Attribution/Licensing

Graphics assets (font.png, font.xcf, man.aseprite, map.aseprite, map.png, rawmap.aseprite, tree.aseprite) are [CC0 licensed](./license.assets.md), and authored by me.

[DeltaBlock](https://ggbot.itch.io/delta-block-font) is [CC0](./license.assets.md) by GGBotNet

Code assets (*.rs, Cargo files, etc.) are [GPLv3](./license.code.md), authored by me and are linked to the [agb.rs](https://github.com/agbrs/agb) library/framework, which is MPL2.0 license and therefore compatible with GPLv3.

## Acknowledgements

Many thanks to:

- The agb.rs team for the amazing foundation, allowing me to develop for the GBA in a language I feel comfortable in.
- GGBotNet for the DeltaBlock font used in the menu
- cearn and the gbadev team for the TONC GBA Guide, an invaluable contribution to the GBA community in general