# Torizon

Torizon is a Rust client and launcher fork with offline profiles, its own branding,
and no partner advertisements, shop, community promotions or upstream telemetry.

The current native game targets **Minecraft Java 26.3**. Selecting a Horizon release
does not switch the Minecraft version. Native 26.2 and older game engines are not
implemented in this fork; importing older worlds upgrades them to 26.3.

On Windows, extract the complete release ZIP into a writable folder and run
**Torizon.exe**. Keep `runtime/` beside it. User data stays in `data/` and `game/`.
The separate EXE download is a launcher replacement; a first installation needs the ZIP.
Choose the folder of your existing Minecraft 26.3 installation to prepare resources.
Fabric/TLauncher folders are supported when their client JAR matches the pinned game.
Minecraft's JARs, textures, sounds and prepared resources are not distributed here.

Offline profiles work in single player and on servers permitting offline login.
Servers enforcing authenticated sessions still require a valid account.

The renderer and integrated server run on separate threads. Chunk generation, lighting,
meshing, chunk encoding and distant terrain share a worker pool. The game already selects
all CPU threads except one automatically; Graphics settings also expose the worker count.
Ordered world simulation still has sequential stages. More workers do not guarantee more FPS.

Built from [Horizon](https://github.com/horizon-rust/horizon), under the MIT license.
Original copyright notices and third party license notices are retained in `LICENSE`
and `THIRD_PARTY_NOTICES.md`.
