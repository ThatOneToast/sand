// Decode Sand's exported fixtures with the unmodified Minecraft 26.2 codecs.
// Compile/run with the extracted server jar and its libraries on the classpath.
import com.google.gson.JsonParser;
import com.mojang.serialization.JsonOps;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.data.registries.VanillaRegistries;
import net.minecraft.resources.RegistryOps;
import net.minecraft.sounds.SoundEvent;
import net.minecraft.world.item.Instrument;
import net.minecraft.world.item.JukeboxSong;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Optional;

class SoundEventResourceProbe {
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        var ops = RegistryOps.create(JsonOps.INSTANCE, VanillaRegistries.createLookup());
        for (String name : new String[]{"reference", "inline", "fixed", "zero", "negative"}) {
            String expectedId = name.equals("reference") ? "minecraft:block.note_block.harp" : "sand_audit:custom_track";
            Optional<Float> expectedRange = switch (name) {
                case "fixed" -> Optional.of(32.0f);
                case "zero" -> Optional.of(0.0f);
                case "negative" -> Optional.of(-1.0f);
                default -> Optional.empty();
            };
            for (String kind : new String[]{"instrument", "jukebox_song"}) {
                var path = Path.of(args[0], "data", "sand_audit", kind, "sound", name + ".json");
                var json = JsonParser.parseString(Files.readString(path));
                SoundEvent sound;
                boolean reference;
                if (kind.equals("instrument")) {
                    var instrument = Instrument.DIRECT_CODEC.parse(ops, json).getOrThrow();
                    if (instrument.range() != 256.0f) throw new AssertionError("outer instrument range: " + path);
                    sound = instrument.soundEvent().value();
                    reference = instrument.soundEvent().unwrapKey().isPresent();
                } else {
                    var song = JukeboxSong.DIRECT_CODEC.parse(ops, json).getOrThrow();
                    if (song.lengthInSeconds() != 10.0f || song.comparatorOutput() != 0) throw new AssertionError("song properties: " + path);
                    sound = song.soundEvent().value();
                    reference = song.soundEvent().unwrapKey().isPresent();
                }
                if (!sound.location().toString().equals(expectedId) || !sound.fixedRange().equals(expectedRange)
                    || reference != name.equals("reference")) throw new AssertionError(path + ": " + sound);
                System.out.println("PASS " + kind + "/sound/" + name + " reference=" + reference + " " + sound);
            }
        }
    }
}
