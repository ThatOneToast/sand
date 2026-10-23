# Shared sound-event resource evidence

The typed `sand-vanilla-audit` export passed Minecraft 26.2 startup, ten item
registry resolutions, reload, and clean shutdown on October 23, 2026. The native
Java codecs also decoded all ten exported files and verified their holder and
range properties. The tested source was clean commit
`3948dcb02d2ac4bb22be90e02269866d0244729f`; the subsequent evidence commit
does not change the tested implementation, fixtures, or harnesses.

[Machine-readable evidence](sound-event-evidence.json) records that exact head,
fixture and harness hashes, bootstrap and extracted server JAR hashes, every
sound resource's JSON and hash, a canonical whole-pack file-map hash, and codec
probe output. The pack hash includes all 186 files present in the exported pack.

## Assertions

Both `Instrument` and `JukeboxSong` cover a registered vanilla reference and
inline `sand_audit:custom_track` definitions with omitted, 32, zero, and negative
one ranges. The server resolves each resource through its corresponding item
component. The Java probe checks registered versus direct holders, sound IDs,
and exact optional fixed ranges. Every instrument retains its separate outer
range of 256; songs retain length 10 and comparator output zero.

These checks prove resource loading and codec semantics. They do not prove
audio playback or provide the audio assets for a custom resource pack. The
earlier handwritten codec experiment is supporting research, not this export
proof. An initial harness run incorrectly supplied an object to the 26.2
`jukebox_playable` item component; its native codec requires a string holder.
The committed harness correction passed against unchanged exported files.

## Reproduction

Use Java 25, a Minecraft 26.2 server bootstrap JAR, and a fresh disposable output
directory. From the repository root:

```sh
export CARGO_BUILD_JOBS=2 CARGO_HTTP_MULTIPLEXING=false
export SAND_MC_VERSION=26.2 SAND_STRICT_CODEGEN=1
export CARGO_TARGET_DIR="$PWD/target"
(cd sand-vanilla-audit && cargo run --manifest-path ../sand-cli/Cargo.toml -- build)
python3 scripts/validate_sound_events.py --version 26.2 \
  --pack sand-vanilla-audit/dist/sand_audit \
  --jar "$HOME/.sand/cache/26.2/server.jar" \
  --output /tmp/sand-sound-proof --timeout 180
```

The server harness binds to loopback and inherits the standard reload harness's
error detection and shutdown checks. Its explicit output directory is replaced
when starting a run. To decode those same exported files using the server and
libraries extracted by that run:

```sh
mkdir -p /tmp/sand-sound-proof/probe
SOUND_CP=$(find /tmp/sand-sound-proof/server/versions /tmp/sand-sound-proof/server/libraries -name '*.jar' -print | paste -sd ':' -)
javac -cp "$SOUND_CP" -d /tmp/sand-sound-proof/probe \
  scripts/mc_validation/SoundEventResourceProbe.java
java -cp "/tmp/sand-sound-proof/probe:$SOUND_CP" SoundEventResourceProbe \
  "$PWD/sand-vanilla-audit/dist/sand_audit"
```

Focused Rust validation separately passed six sound-definition tests, all 53
compiler record tests, the facade's reference/inline and namespace A/B/A export
test, and 17 API contract/manifest/surface assertions. Normal and genuine
failing-target placeholder checks passed with canonically generated API reports.
Full workspace validation and CI are separate gates; this evidence does not
stand in for them.
