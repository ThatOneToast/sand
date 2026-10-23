// Decode exported Sand biomes, then inspect and re-encode the actual 26.2 attribute.
// This proves that the native codec retains the settings, not just that a pack loads.
import com.google.gson.JsonParser;
import com.mojang.serialization.JsonOps;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.data.registries.VanillaRegistries;
import net.minecraft.resources.RegistryOps;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.attribute.AmbientSounds;
import net.minecraft.world.attribute.AmbientMoodSettings;
import net.minecraft.world.attribute.AmbientAdditionsSettings;
import net.minecraft.world.attribute.EnvironmentAttributes;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Optional;

class BiomeAmbientSoundProbe {
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();
        if (!SharedConstants.getCurrentVersion().name().equals("26.2")) throw new AssertionError("expected 26.2");
        Bootstrap.bootStrap();
        var ops = RegistryOps.create(JsonOps.INSTANCE, VanillaRegistries.createLookup());
        for (String name : new String[]{"reference","inline","fixed","zero","negative"}) {
            var path = Path.of(args[0],"data","sand_audit","worldgen","biome","ambient",name+".json");
            var json = JsonParser.parseString(Files.readString(path));
            var biome = Biome.DIRECT_CODEC.parse(ops,json).getOrThrow();
            if (!biome.getAttributes().contains(EnvironmentAttributes.AMBIENT_SOUNDS)) throw new AssertionError("missing attribute: "+path);
            var sounds = biome.getAttributes().applyModifier(EnvironmentAttributes.AMBIENT_SOUNDS,AmbientSounds.EMPTY);
            var expectedRange = switch(name) {case "fixed" -> Optional.of(32f); case "zero" -> Optional.of(0f); case "negative" -> Optional.of(-1f); default -> Optional.<Float>empty();};
            String expectedId = name.equals("reference") ? "minecraft:ambient.cave" : "sand_audit:custom_track";
            var mood = sounds.mood().orElseThrow();
            if (mood.tickDelay()!=6000 || mood.blockSearchExtent()!=8 || mood.soundPositionOffset()!=2.0) throw new AssertionError("mood settings");
            if (sounds.additions().size()!=2 || sounds.additions().get(0).tickChance()!=-1 || sounds.additions().get(1).tickChance()!=2) throw new AssertionError("additions");
            for (var holder : java.util.List.of(sounds.loop().orElseThrow(), mood.soundEvent(), sounds.additions().get(0).soundEvent(),sounds.additions().get(1).soundEvent())) {
                if (holder.unwrapKey().isPresent()!=name.equals("reference") || !holder.value().location().toString().equals(expectedId) || !holder.value().fixedRange().equals(expectedRange)) throw new AssertionError("holder: "+path);
            }
            var encoded=Biome.DIRECT_CODEC.encodeStart(ops,biome).getOrThrow().getAsJsonObject();
            if (!encoded.getAsJsonObject("attributes").has("minecraft:audio/ambient_sounds")) throw new AssertionError("attribute lost on encoding");
            System.out.println("PASS "+name+" "+AmbientSounds.CODEC.encodeStart(ops,sounds).getOrThrow());
        }
        // Codec boundaries and defaults, independent of runtime playback semantics.
        var mood = AmbientMoodSettings.CODEC.parse(ops,JsonParser.parseString("{\"sound\":\"minecraft:ambient.cave\",\"tick_delay\":-2147483648,\"block_search_extent\":2147483647,\"offset\":-2}")).getOrThrow();
        if (mood.tickDelay()!=Integer.MIN_VALUE || mood.blockSearchExtent()!=Integer.MAX_VALUE || mood.soundPositionOffset()!=-2) throw new AssertionError("signed codec fields");
        var empty=AmbientSounds.CODEC.parse(ops,JsonParser.parseString("{}")).getOrThrow();
        if (empty.loop().isPresent() || empty.mood().isPresent() || !empty.additions().isEmpty()) throw new AssertionError("defaults");
        var compact=AmbientSounds.CODEC.parse(ops,JsonParser.parseString("{\"additions\":{\"sound\":\"minecraft:ambient.cave\",\"tick_chance\":2}}")).getOrThrow();
        if (compact.additions().size()!=1 || compact.additions().getFirst().tickChance()!=2) throw new AssertionError("compact additions");
        for (String missing : new String[]{"sound","tick_delay","block_search_extent","offset"}) {
            var value=JsonParser.parseString("{\"sound\":\"minecraft:ambient.cave\",\"tick_delay\":1,\"block_search_extent\":1,\"offset\":0}").getAsJsonObject();
            value.remove(missing);
            if (AmbientMoodSettings.CODEC.parse(ops,value).isSuccess()) throw new AssertionError("missing "+missing);
        }
        System.out.println("PASS native signed bounds, defaults, compact additions, required fields");
    }
}
