# Mount API

> The main entry point for SparkTUI applications.

## Import

```ts
import {
  mount,
  mountSync,
  mountForTest,
  isMounted,
  getRenderMode
} from 'spark-tui';

import type { MountOptions, MountHandle, MountRenderMode } from 'spark-tui';
```

## Overview

The mount API handles everything needed to run a SparkTUI application:
- Bridge initialization (SharedArrayBuffer + reactive arrays)
- Rust engine loading and initialization
- Event listener startup (truly reactive, not polling)
- Terminal size detection
- Render mode configuration
- Clean unmount with full cleanup

SparkTUI is **purely reactive**: there are no loops, no polling, no fixed FPS. Changes propagate through the dependency graph automatically.

## `mount()` Function

The primary entry point for most applications. Blocks until the app exits.

### Signature

```ts
async function mount(app: () => void, options?: MountOptions): Promise<void>
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `app` | `() => void` | The app function that creates the UI |
| `options` | `MountOptions` | Optional configuration |

### Example

```ts
import { mount, box, text } from 'spark-tui';

await mount(() => {
  box({
    width: '100%',
    height: '100%',
    children: () => {
      text({ content: 'Hello, SparkTUI!' });
    }
  });
});
// Execution continues here after user exits (Ctrl+C)
```

## `mountSync()` Function

Synchronous mount that returns a handle for manual control. For power users and tests.

### Signature

```ts
function mountSync(app: () => void, options?: MountOptions): MountHandle
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `app` | `() => void` | The app function that creates the UI |
| `options` | `MountOptions` | Optional configuration |

### Returns

A `MountHandle` object for controlling the mounted application.

### Example

```ts
import { mountSync, box, text } from 'spark-tui';

const app = mountSync(() => {
  box({ children: () => text({ content: 'Hello!' }) });
});

// Do something with the running app
console.log('Terminal size:', app.buffer);

// Wait for user to exit
await app.waitForExit();
```

## `mountForTest()` Function

Convenience function for testing. Automatically uses `noopNotifier: true` to run without the Rust engine.

### Signature

```ts
function mountForTest(
  app: () => void,
  options?: Omit<MountOptions, 'noopNotifier'>
): MountHandle
```

### Example

```ts
import { mountForTest, box, text } from 'spark-tui';

// Mount for testing (no Rust engine needed)
const app = mountForTest(() => {
  box({ width: 50, height: 10, children: () => {
    text({ content: 'Test content' });
  }});
});

// Inspect state
// ... run assertions ...

// Clean up
app.unmount();
```

## `MountOptions` Interface

Configuration options for mount functions.

```ts
interface MountOptions {
  /** Render mode: fullscreen (default), inline, or append */
  mode?: MountRenderMode;

  /** Terminal width (auto-detected if not specified) */
  width?: number;

  /** Terminal height (auto-detected if not specified) */
  height?: number;

  /** Disable Ctrl+C exit handling (default: enabled) */
  disableCtrlC?: boolean;

  /** Disable Tab focus navigation (default: enabled) */
  disableTabNavigation?: boolean;

  /** Disable mouse support (default: enabled) */
  disableMouse?: boolean;

  /** Callback when app is unmounted */
  onUnmount?: () => void;

  /** Use noop notifier (for testing without Rust) */
  noopNotifier?: boolean;

  /** Maximum number of nodes (default: 10,000) */
  maxNodes?: number;

  /** Text pool size in bytes (default: 10MB) */
  textPoolSize?: number;
}
```

### Options Details

#### `mode`

Type: `'fullscreen' | 'inline' | 'append'`

Default: `'fullscreen'`

Controls how the UI is rendered to the terminal:

| Mode | Description |
|------|-------------|
| `fullscreen` | Uses alternate screen buffer, clears screen, full terminal control |
| `inline` | Renders within terminal flow, respects scroll position |
| `append` | Appends output without clearing, good for logs/status |

#### `width` / `height`

Type: `number`

Override terminal dimensions. If not specified, dimensions are auto-detected from `process.stdout`.

#### `disableCtrlC`

Type: `boolean`

Default: `false`

When `true`, Ctrl+C will not automatically exit the application. You must handle exit manually.

#### `disableTabNavigation`

Type: `boolean`

Default: `false`

When `true`, Tab key will not cycle focus between focusable components.

#### `disableMouse`

Type: `boolean`

Default: `false`

When `true`, mouse events are not processed.

#### `onUnmount`

Type: `() => void`

Callback invoked when the application is unmounted.

#### `noopNotifier`

Type: `boolean`

Default: `false`

When `true`, runs without the Rust engine. Useful for testing.

#### `maxNodes`

Type: `number`

Default: `10000`

Maximum number of UI components. Increase for very large UIs.

#### `textPoolSize`

Type: `number`

Default: `10485760` (10 MB)

Size of the text pool in bytes. Increase if your app has lots of text content.

## `MountHandle` Interface

Handle returned by `mountSync()` and `mountForTest()` for controlling a mounted application.

```ts
interface MountHandle {
  /** Unmount the app and clean up */
  unmount(): void;

  /** Get the shared buffer for direct access */
  buffer: SharedBuffer;

  /** Get the Rust engine for direct access */
  engine: SparkEngine;

  /** Switch render mode at runtime */
  setMode(mode: MountRenderMode): void;

  /** Get current render mode */
  getMode(): MountRenderMode;

  /** Block until the app exits */
  waitForExit(): Promise<void>;
}
```

### Methods

#### `unmount()`

Unmounts the application and performs cleanup:
- Stops the event listener
- Cleans up all event handlers
- Runs component cleanup functions
- Closes the Rust engine
- Resets the bridge

#### `waitForExit()`

Returns a Promise that resolves when the app exits (via Ctrl+C, 'q', or programmatic unmount).

#### `setMode(mode)`

Change the render mode at runtime:

```ts
const app = mountSync(() => { /* ... */ }, { mode: 'fullscreen' });

// Later, switch to inline mode
app.setMode('inline');
```

#### `getMode()`

Get the current render mode:

```ts
const currentMode = app.getMode();  // 'fullscreen' | 'inline' | 'append'
```

## `MountRenderMode` Type

```ts
type MountRenderMode = 'fullscreen' | 'inline' | 'append';
```

## Helper Functions

### `isMounted()`

Check if SparkTUI is currently mounted:

```ts
import { isMounted } from 'spark-tui';

if (isMounted()) {
  console.log('App is running');
}
```

### `getRenderMode()`

Get the current render mode:

```ts
import { getRenderMode } from 'spark-tui';

const mode = getRenderMode();  // 'fullscreen' | 'inline' | 'append'
```

## Examples

### Simple Fullscreen App

```ts
import { mount, box, text } from 'spark-tui';

await mount(() => {
  box({
    width: '100%',
    height: '100%',
    justifyContent: 'center',
    alignItems: 'center',
    children: () => {
      text({ content: 'Press Ctrl+C to exit' });
    }
  });
});
```

### Inline Status Display

```ts
import { mount, box, text } from 'spark-tui';

await mount(() => {
  box({
    border: 1,
    padding: 1,
    children: () => {
      text({ content: 'Processing: 50%' });
    }
  });
}, { mode: 'inline' });
```

### Custom Exit Handling

```ts
import { mountSync, box, text } from 'spark-tui';
import { state } from '@rlabs-inc/signals';

const running = state(true);

const app = mountSync(() => {
  box({
    focusable: true,
    onKey: (e) => {
      if (e.key === 'q') {
        running.value = false;
        app.unmount();
      }
    },
    children: () => {
      text({ content: "Press 'q' to quit" });
    }
  });
}, { disableCtrlC: true });

await app.waitForExit();
console.log('Goodbye!');
```

### Testing Pattern

```ts
import { mountForTest, box, text } from 'spark-tui';
import { describe, test, expect } from 'bun:test';

describe('MyComponent', () => {
  test('renders correctly', () => {
    const app = mountForTest(() => {
      box({
        width: 20,
        height: 5,
        children: () => {
          text({ content: 'Hello' });
        }
      });
    });

    // Access buffer for assertions
    const nodeCount = app.buffer.view.getUint32(4, true);
    expect(nodeCount).toBeGreaterThan(0);

    app.unmount();
  });
});
```

### Power User: Direct Buffer Access

```ts
import { mountSync, box, text } from 'spark-tui';
import { getNodeCount, getComputedWidth } from 'spark-tui/bridge/shared-buffer';

const app = mountSync(() => {
  box({
    width: '50%',
    children: () => {
      text({ content: 'Measuring...' });
    }
  });
});

// Access computed layout values
const nodeCount = getNodeCount(app.buffer);
console.log(`Total nodes: ${nodeCount}`);

for (let i = 0; i < nodeCount; i++) {
  const width = getComputedWidth(app.buffer, i);
  console.log(`Node ${i} computed width: ${width}`);
}

await app.waitForExit();
```

## Lifecycle

```
mount() called
    |
    v
Bridge initialized (SharedArrayBuffer created)
    |
    v
Rust engine loaded and initialized
    |
    v
Event listener started (worker-based, non-blocking)
    |
    v
App function executed (UI tree created)
    |
    v
[App is running - reactive updates happen automatically]
    |
    v
Exit triggered (Ctrl+C, 'q', or unmount())
    |
    v
Cleanup:
  - Event listener stopped
  - Component cleanup functions run
  - Rust engine closed
  - Bridge reset
    |
    v
mount() Promise resolves
```

## Error Handling

### Already Mounted

```ts
import { mount } from 'spark-tui';

await mount(() => { /* app 1 */ });
await mount(() => { /* app 2 */ });  // Throws: "SparkTUI is already mounted"
```

To run multiple apps sequentially, ensure the first unmounts before starting the second.

### Engine Init Failure

If the Rust engine fails to initialize, `mount()` throws with the error code:

```ts
try {
  await mount(() => { /* ... */ });
} catch (e) {
  console.error('Failed to start:', e.message);
}
```

## See Also

- [Getting Started](/docs/getting-started.md)
- [Components](/docs/components/)
- [Events](/docs/events/)
- [Testing](/docs/testing/)
