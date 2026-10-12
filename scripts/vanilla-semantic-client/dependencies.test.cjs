const assert = require('node:assert/strict')
const { createRequire } = require('node:module')
const test = require('node:test')

test('the semantic client dependencies remain loadable from CommonJS', () => {
  assert.equal(typeof require('mineflayer').createBot, 'function')
  assert.equal(typeof require('@azure/msal-node').PublicClientApplication, 'function')
  assert.equal(typeof require('yggdrasil'), 'function')
})

for (const consumer of ['@azure/msal-node', 'yggdrasil']) {
  test(`${consumer} resolves UUID with checked output-buffer bounds`, () => {
    const uuid = createRequire(require.resolve(consumer))('uuid')
    assert.ok(uuid.validate(uuid.v4()))
    for (const generate of [
      buffer => uuid.v3('sand', uuid.v3.DNS, buffer, 4),
      buffer => uuid.v5('sand', uuid.v5.DNS, buffer, 4),
      buffer => uuid.v6({}, buffer, 4)
    ]) {
      const buffer = new Uint8Array(8).fill(0xaa)
      assert.throws(() => generate(buffer), RangeError)
      assert.deepEqual(buffer, new Uint8Array(8).fill(0xaa))
    }
  })
}
