import { initBridgeAoS } from '../ts/bridge'
import { box } from '../ts/primitives/box'
import { text } from '../ts/primitives/text'
import { loadEngine } from '../ts/bridge/ffi'
import { setTerminalSize } from '../ts/bridge/shared-buffer-aos'
import { startEventListener, registerExitHandler, registerKeyHandler } from '../ts/engine/events'

console.log("🚀 Starting Hello World AoS...")

// 1. Initialize AoS bridge
const bridge = initBridgeAoS()
console.log("✅ Bridge initialized")

// 2. Create UI
console.log("📦 Creating UI components...")
const root = box({
  width: 40,
  height: 10,
  border: 1,
  children: [
    text({ content: 'Hello, SparkTUI!' }),
    text({ content: 'Press Ctrl+C to exit' })
  ]
})
console.log("✅ UI created")

// 3. Set terminal size in header (initial guess)
setTerminalSize(bridge.buf, 80, 24)

// 4. Start Event Listener
startEventListener(bridge.buf)
registerExitHandler(() => {
  console.log("👋 Exit requested")
  process.exit(0)
})
registerKeyHandler(0, (e) => {
    console.log("🔑 Key event:", e)
})
console.log("✅ Event listener started")

// 5. Load and start Rust engine
const engine = loadEngine()
const result = engine.init(bridge.buf.buffer as any, bridge.buf.buffer.byteLength)

if (result !== 0) {
  console.error(`❌ Failed to start engine: code ${result}`)
  process.exit(1)
}
console.log("✅ Engine started (Rust thread running)")

// Keep alive
setInterval(() => {}, 1000)
