/** Source-launch against the verified vanilla server and library jars.
 * Validates CLIENT_TYPE metadata decoding without claiming client rendering. */
class MetadataProbe {
    public static void main(String[] args) throws Exception {
        net.minecraft.SharedConstants.tryDetectVersion();
        net.minecraft.server.Bootstrap.bootStrap();
        var json = com.google.gson.JsonParser.parseString(
            java.nio.file.Files.readString(java.nio.file.Path.of(args[0]))
        ).getAsJsonObject().get("pack");
        var metadata = net.minecraft.server.packs.metadata.pack.PackMetadataSection.CLIENT_TYPE
            .codec().parse(com.mojang.serialization.JsonOps.INSTANCE, json).getOrThrow();
        var range = metadata.supportedFormats();
        int major = Integer.parseInt(args[1]);
        int minor = Integer.parseInt(args[2]);
        if (range.minInclusive().major() != major || range.minInclusive().minor() != minor
            || range.maxInclusive().major() != major || range.maxInclusive().minor() != minor)
            throw new AssertionError(range);
        System.out.println("Native client pack-metadata codec retained exact resource format " + range);
    }
}
