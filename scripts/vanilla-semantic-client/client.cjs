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
let whileSneakingMatches = 0;
let afterAnyMatches = 0;
let afterAllMatches = 0;
let withinMatches = 0;
let interactionSequence = 0;
let completed = false;

bot.on("messagestr", (message) => {
  if (message.includes("__SAND_SEMANTIC_PLACED__")) placedMatches += 1;
  if (message.includes("__SAND_SEMANTIC_ITEM_USED__")) itemUsedMatches += 1;
  if (message.includes("__SAND_SEMANTIC_WHILE_SNEAKING__")) whileSneakingMatches += 1;
  if (message.includes("__SAND_SEMANTIC_AFTER_ANY__")) afterAnyMatches += 1;
  if (message.includes("__SAND_SEMANTIC_AFTER_ALL__")) afterAllMatches += 1;
  if (message.includes("__SAND_SEMANTIC_WITHIN__")) withinMatches += 1;
});

function fail(message) {
  throw new Error(
    `${message} (placed=${placedMatches}, item_used=${itemUsedMatches}, while_sneaking=${whileSneakingMatches}, after_any=${afterAnyMatches}, after_all=${afterAllMatches}, within=${withinMatches})`,
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

async function assertWhileSneaking(expected, label) {
  await waitTicks(5);
  if (whileSneakingMatches !== expected) {
    fail(`${label}: expected while_sneaking=${expected}`);
  }
}

async function assertMultiParent(expectedAny, expectedAll, label) {
  await waitTicks(5);
  if (afterAnyMatches !== expectedAny || afterAllMatches !== expectedAll) {
    fail(`${label}: expected after_any=${expectedAny}, after_all=${expectedAll}`);
  }
}

async function assertWithin(expected, label) {
  await waitTicks(5);
  if (withinMatches !== expected) {
    fail(`${label}: expected within=${expected}`);
  }
}

// Fires a scoreboard-driven stimulus without the standard `command()` 20-tick
// pacing, so two stimuli can land close enough together to exercise a small
// (5-tick) bounded correlation window.
async function tightCommand(commandText, ticks) {
  bot.chat(`/${commandText}`);
  await waitTicks(ticks);
}

function setSneaking(state) {
  // Send the exact 1.21.4 client action packet. Using a raw packet here keeps
  // the semantic stimulus unambiguous and independent of Mineflayer physics.
  bot._client.write("entity_action", {
    entityId: bot.entity.id,
    actionId: state ? 0 : 1,
    jumpBoost: 0,
  });
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
    // Mineflayer may briefly report an interpolated pre-teleport position;
    // the packet-driven assertions below still target and verify exact blocks.
    const atTestPosition = async () =>
      bot.entity.position.distanceTo(new Vec3(baseX + 0.5, 79, baseZ + 0.5)) < 1.5;
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

    await command("scoreboard players set @s sand_sem_occ 0");
    await command("scoreboard players set @s sand_sem_prev 0");
    setSneaking(true);
    await waitTicks(10);
    await assertWhileSneaking(0, "state becoming true without a parent occurrence must not fire");
    await command("scoreboard players add @s sand_sem_occ 1");
    await assertWhileSneaking(1, "first parent observation while true must fire once");
    await command("scoreboard players add @s sand_sem_occ 1");
    await assertWhileSneaking(2, "state remaining true must allow a later parent occurrence");

    setSneaking(false);
    await waitTicks(10);
    await command("scoreboard players add @s sand_sem_occ 1");
    await assertWhileSneaking(2, "state becoming false must stop child dispatch");

    setSneaking(true);
    await waitTicks(10);
    await command("scoreboard players add @s sand_sem_occ 1");
    await assertWhileSneaking(3, "re-entering true state must allow repeated firing");
    setSneaking(false);

    await command("scoreboard players set @s sand_mp_a 0");
    await command("scoreboard players set @s sand_mp_ap 0");
    await command("scoreboard players set @s sand_mp_b 0");
    await command("scoreboard players set @s sand_mp_bp 0");
    await assertMultiParent(0, 0, "neither multi-parent occurrence must not fire");

    await command("function sand_audit:semantic_multi_fire_a");
    await assertMultiParent(1, 0, "parent A alone must satisfy any but not all");
    await waitTicks(10);
    await assertMultiParent(1, 0, "parent A must not remain visible in a later cycle");

    await command("function sand_audit:semantic_multi_fire_a");
    await assertMultiParent(
      2,
      0,
      "repeating parent A must not substitute for missing parent B",
    );

    await command("function sand_audit:semantic_multi_fire_b");
    await assertMultiParent(3, 0, "parent B alone must satisfy any but not all");
    await waitTicks(10);
    await assertMultiParent(3, 0, "parent B must not remain visible in a later cycle");

    await command("function sand_audit:semantic_multi_fire_ab");
    await assertMultiParent(
      4,
      1,
      "both parents in one cycle must satisfy all and coalesce any to one dispatch",
    );
    await waitTicks(10);
    await assertMultiParent(4, 1, "both-parent marks must reset before the next cycle");

    await command("function sand_audit:semantic_multi_fire_ba");
    await assertMultiParent(
      5,
      2,
      "reverse atomic parent order must preserve any/all behavior",
    );

    // Phase 5 (#240): bounded `.within(...)` correlation. SemanticOccurrence
    // is the current trigger (sand_sem_occ), SemanticMultiParentA is the
    // 5-tick bounded prior event (sand_mp_a). Reset both delta-tracked
    // parents and let any lingering age from the after_any/after_all block
    // above decay well past the window before asserting a clean baseline.
    await command("scoreboard players set @s sand_sem_occ 0");
    await command("scoreboard players set @s sand_sem_prev 0");
    await command("scoreboard players set @s sand_mp_a 0");
    await command("scoreboard players set @s sand_mp_ap 0");
    await waitTicks(30);

    await tightCommand("scoreboard players add @s sand_sem_occ 1", 5);
    await assertWithin(0, "current fires without a recent prior occurrence must not match");

    await tightCommand("scoreboard players add @s sand_mp_a 1", 2);
    await tightCommand("scoreboard players add @s sand_sem_occ 1", 5);
    await assertWithin(1, "prior fired one to two ticks before current must match within the window");

    await tightCommand("scoreboard players add @s sand_mp_a 1", 2);
    await tightCommand("scoreboard players add @s sand_sem_occ 1", 5);
    await assertWithin(
      2,
      "a later prior occurrence refreshes the window for a second current firing",
    );

    await waitTicks(30);
    await tightCommand("scoreboard players add @s sand_sem_occ 1", 5);
    await assertWithin(
      2,
      "current firing long after the last prior occurrence (window expired) must not match",
    );

    console.log(
      `PASSED semantic gameplay: placed=${placedMatches} item_used=${itemUsedMatches} while_sneaking=${whileSneakingMatches} after_any=${afterAnyMatches} after_all=${afterAllMatches} within=${withinMatches}`,
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
