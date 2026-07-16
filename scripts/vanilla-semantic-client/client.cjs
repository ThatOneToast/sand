"use strict";

const mineflayer = require("mineflayer");
const { Vec3 } = require("vec3");

const host = process.env.SAND_SERVER_HOST || "127.0.0.1";
const port = Number(process.env.SAND_SERVER_PORT);
const version = process.env.SAND_MC_VERSION;
const username = "SandAuditBot";
const baseX = 0;
const baseZ = 0;

if (!port || version !== "1.21.4") {
  throw new Error(`semantic client requires a 1.21.4 server port; got ${version}:${port}`);
}

const bot = mineflayer.createBot({ host, port, username, version, auth: "offline" });
let placedMatches = 0;
let itemUsedMatches = 0;
let interactionSequence = 0;
let completed = false;

bot.on("messagestr", (message) => {
  if (message.includes("__SAND_SEMANTIC_PLACED__")) placedMatches += 1;
  if (message.includes("__SAND_SEMANTIC_ITEM_USED__")) itemUsedMatches += 1;
});

function fail(message) {
  throw new Error(
    `${message} (placed=${placedMatches}, item_used=${itemUsedMatches})`,
  );
}

async function waitTicks(ticks = 10) {
  await new Promise((resolve) => setTimeout(resolve, ticks * 50));
}

async function command(commandText) {
  bot.chat(`/${commandText}`);
  await waitTicks(20);
}

async function commandUntil(commandText, predicate, label) {
  for (let attempt = 1; attempt <= 5; attempt += 1) {
    await command(commandText);
    if (await predicate()) return;
  }
  fail(`setup command did not establish ${label}: /${commandText}`);
}

async function give(itemCommand, itemName, count) {
  await commandUntil(
    `item replace entity @s weapon.mainhand with ${itemCommand} ${count}`,
    async () => {
      const item = bot.heldItem;
      return item?.name === itemName && item.count === count;
    },
    `${count} ${itemName} in the main hand`,
  );
  await waitTicks(3);
}

async function placeAt(x, z) {
  const floor = bot.blockAt(new Vec3(x, 78, z));
  if (!floor || floor.name === "air") fail(`missing placement floor at ${x},78,${z}`);
  await bot.lookAt(floor.position.offset(0.5, 1, 0.5), false);
  interactionSequence += 1;
  bot._client.write("block_place", {
    location: floor.position,
    direction: 1,
    hand: 0,
    cursorX: 0.5,
    cursorY: 1,
    cursorZ: 0.5,
    insideBlock: false,
    sequence: interactionSequence,
    worldBorderHit: false,
  });
  bot.swingArm();
  await waitTicks(12);
  const placed = bot.blockAt(new Vec3(x, 79, z));
  if (!placed || placed.name === "air") fail(`server rejected placement at ${x},79,${z}`);
}

async function useAt(x, z) {
  const block = bot.blockAt(new Vec3(x, 79, z));
  if (!block || block.name === "air") fail(`missing use target at ${x},79,${z}`);
  await bot.activateBlock(block);
  await waitTicks(12);
}

async function assertCounts(expectedPlaced, expectedItemUsed, label) {
  await waitTicks(5);
  if (placedMatches !== expectedPlaced || itemUsedMatches !== expectedItemUsed) {
    fail(`${label}: expected placed=${expectedPlaced}, item_used=${expectedItemUsed}`);
  }
}

bot.once("spawn", async () => {
  try {
    await commandUntil(
      `gamemode survival @s`,
      async () => bot.game.gameMode === "survival",
      "survival mode",
    );
    for (let attempt = 0; attempt < 3; attempt += 1) {
      await command(`fill ${baseX - 5} 79 ${baseZ - 5} ${baseX + 5} 85 ${baseZ + 5} air`);
      await command(`fill ${baseX - 5} 78 ${baseZ - 5} ${baseX + 5} 78 ${baseZ + 5} stone`);
    }
    const atTestPosition = async () =>
      bot.entity.position.distanceTo(new Vec3(baseX + 0.5, 79, baseZ + 0.5)) < 0.25;
    await commandUntil(
      `tp @s ${baseX + 0.5} 79 ${baseZ + 0.5}`,
      atTestPosition,
      "test position",
    );
    await commandUntil(
      `fill ${baseX - 5} 79 ${baseZ - 5} ${baseX + 5} 85 ${baseZ + 5} air`,
      async () => bot.blockAt(new Vec3(baseX, 79, baseZ + 2))?.name === "air",
      "an empty placement arena",
    );
    await commandUntil(
      `fill ${baseX - 5} 78 ${baseZ - 5} ${baseX + 5} 78 ${baseZ + 5} stone`,
      async () => bot.blockAt(new Vec3(baseX, 78, baseZ + 2))?.name === "stone",
      "the stone placement floor",
    );
    await commandUntil(
      `tp @s ${baseX + 0.5} 79 ${baseZ + 0.5}`,
      atTestPosition,
      "test position after arena setup",
    );

    await give("minecraft:dirt", "dirt", 1);
    await placeAt(baseX, baseZ + 2);
    await assertCounts(0, 0, "unrelated block must not match placement");

    await give("minecraft:white_wool", "white_wool", 1);
    await placeAt(baseX + 1, baseZ + 2);
    await assertCounts(0, 0, "ordinary base item must not match marked placement");

    await give(
      "minecraft:white_wool[minecraft:custom_data={elevator:1b}]",
      "white_wool",
      2,
    );
    await placeAt(baseX - 1, baseZ + 2);
    await assertCounts(1, 0, "marked placement must fire exactly once");
    await placeAt(baseX - 2, baseZ + 2);
    await assertCounts(2, 0, "final stack item must fire after revoke/reset");

    await command(`setblock ${baseX + 3} 79 ${baseZ} minecraft:stone`);
    await give(
      "minecraft:honeycomb[minecraft:custom_data={sand_audit_item:1b}]",
      "honeycomb",
      1,
    );
    await useAt(baseX + 3, baseZ);
    await assertCounts(2, 0, "unrelated use target must not match");

    await command(`setblock ${baseX + 2} 79 ${baseZ + 1} minecraft:copper_block`);
    await give("minecraft:honeycomb", "honeycomb", 1);
    await useAt(baseX + 2, baseZ + 1);
    await assertCounts(2, 0, "ordinary item must not match marked item use");

    await command(`setblock ${baseX + 2} 79 ${baseZ + 2} minecraft:copper_block`);
    await command(`setblock ${baseX + 2} 79 ${baseZ + 3} minecraft:copper_block`);
    await give(
      "minecraft:honeycomb[minecraft:custom_data={sand_audit_item:1b}]",
      "honeycomb",
      2,
    );
    await useAt(baseX + 2, baseZ + 2);
    await assertCounts(2, 1, "marked item use must fire exactly once");
    await useAt(baseX + 2, baseZ + 3);
    await assertCounts(2, 2, "final used item must fire after revoke/reset");

    console.log(
      `PASSED semantic gameplay: placed=${placedMatches} item_used=${itemUsedMatches}`,
    );
    completed = true;
    bot.end("semantic audit complete");
  } catch (error) {
    console.error(error.stack || error);
    bot.end("semantic audit failed");
    process.exitCode = 1;
  }
});

bot.on("kicked", (reason) => {
  console.error(`kicked: ${reason}`);
  process.exitCode = 1;
});

bot.on("error", (error) => {
  console.error(error.stack || error);
  process.exitCode = 1;
});

bot.on("end", () => {
  if (!completed) {
    console.error("semantic client ended before every assertion completed");
    process.exitCode = 1;
  }
});
