# Fixed-Position Multiplatform UI Architecture Strategy

> **Survey, reference only** (written by Gemini for a generic Tauri app).
> What Stylus took from it, and what it didn't and why, is in
> `docs/ui-design.md`, "Phones, tablets and pens"; the rollout is
> `docs/roadmap.md` 4a.

## Overview

This strategy outlines an **Adaptive Canvas Architecture** designed for single-codebase applications built with **Rust** and **Tauri (v2)** targeting both desktop and mobile platforms. 

By rejecting continuous responsive web reflows (e.g., fluid `flex-wrap` or cascading media queries), this strategy enables exact pixel-level element positioning customized for specific device categories, while sharing 100% of underlying business logic and state.

---

## 1. Architectural Principles

```
                       ┌─────────────────────────┐
                       │   Rust Backend Core     │
                       │ (Logic, State, Network) │
                       └────────────┬────────────┘
                                    │
                         Tauri IPC / Commands
                                    │
                       ┌────────────┴────────────┐
                       │ Frontend View Dispatch  │
                       └─────┬─────────────┬─────┘
                             │             │
       #[cfg(desktop)]       │             │       #[cfg(mobile)]
  ┌──────────────────────────┴─┐         ┌─┴──────────────────────────┐
  │   Desktop Virtual Canvas   │         │    Mobile Virtual Canvas   │
  │     (Fixed 16:9 Aspect)    │         │   (Fixed 19.5:9 Aspect)    │
  │ Mouse / Keyboard Intents   │         │   Touch / Gesture Intents  │
  └────────────────────────────┘         └────────────────────────────┘
```

1. **Explicit Form-Factor Profiles:** Categorize targets into discrete profiles rather than fluid widths.
2. **Fixed Virtual Canvas:** Elements are placed with absolute or strictly constrained coordinate systems ($X, Y$).
3. **Uniform Proportional Scaling:** Viewport adjustments scale the canvas as a whole rather than reflowing elements relative to each other.
4. **Input Abstraction Layer:** Native platform events map into a unified set of UI intents.

---

## 2. Platform Configuration (Rust & Tauri)

Enforce strict window constraints at the native Tauri layer (`tauri.conf.json`) and via Rust window customization hooks to prevent arbitrary window shapes from disrupting element coordinates.

### Desktop Configuration
* **Fixed Dimensions & Aspect Ratios:** Disable freeform resizing or lock the window to specific, controlled dimensions.
```json
{
  "app": {
    "windows": [
      {
        "title": "Application Canvas",
        "width": 1280,
        "height": 720,
        "minWidth": 1280,
        "minHeight": 720,
        "maxWidth": 1280,
        "maxHeight": 720,
        "resizable": false,
        "fullscreen": false
      }
    ]
  }
}
```

### Mobile Configuration
* **Fullscreen & Orientation Locking:** Enforce strict screen orientation (e.g., Portrait) and hide OS system bars where appropriate to maintain a stable, predictable canvas boundary.

---

## 3. Frontend Virtual Canvas Strategy

### Fixed Resolution & Uniform CSS Scaling

Instead of standard responsive layouts, design each target layout against a single baseline canvas resolution and apply uniform CSS scale transforms based on window dimensions.

#### CSS Virtual Canvas Setup
```css
/* Container forcing absolute canvas size */
.canvas-root {
  width: 1920px;
  height: 1080px;
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%) scale(var(--scale-factor, 1));
  transform-origin: center center;
  overflow: hidden;
}

/* Absolute positioning for UI elements */
.ui-element {
  position: absolute;
  top: 120px;
  left: 450px;
  width: 300px;
  height: 60px;
}
```

#### Dynamic Scale Factor Calculation
Scale the canvas dynamically without altering internal element layout or relative distances:

```typescript
function updateCanvasScale(targetWidth: number, targetHeight: number) {
  const windowWidth = window.innerWidth;
  const windowHeight = window.innerHeight;

  // Scale uniform to fit within current window bounds
  const scaleX = windowWidth / targetWidth;
  const scaleY = windowHeight / targetHeight;
  const scaleFactor = Math.min(scaleX, scaleY);

  document.documentElement.style.setProperty('--scale-factor', scaleFactor.toString());
}

// Initial calculation and listener setup
window.addEventListener('resize', () => updateCanvasScale(1920, 1080));
```

---

## 4. Input Abstraction & Interaction Handling

Input mechanics vary significantly between pointer devices and touch screens. Normalize these interaction methods into abstract UI intents.

### Input Abstraction Matrix

| Feature | Desktop (Mouse / Keyboard) | Mobile (Touch / Gestures) |
| :--- | :--- | :--- |
| **Primary Interaction** | `onClick` (Left click) | `onTouchEnd` / Native tap |
| **Hover States** | Visible (`:hover` styling enabled) | Disabled (Prevent sticky hover states) |
| **Touch Target Size** | Standard (e.g., $24 \text{px} \times 24 \text{px}$) | Enforced Minimum ($48 \text{dp} \times 48 \text{dp}$) |
| **Secondary Input** | Context menus (Right click) | Long press gestures |
| **Keyboard Support** | Native shortcuts, focus states | On-screen virtual keyboard considerations |

### Unified Action Dispatching

```typescript
import { invoke } from '@tauri-apps/api/core';

// Abstracted Action Execution
export function executeAction(actionId: string, payload?: Record<string, unknown>) {
  // Fire Rust backend command
  invoke('handle_ui_intent', { actionId, payload });
}
```

---

## 5. Project Directory Structure

```text
my-tauri-app/
├── src-tauri/                 # Rust Core
│   ├── src/
│   │   ├── lib.rs             # Tauri command handlers & business logic
│   │   └── main.rs            # Application entry point
│   └── tauri.conf.json        # Window & platform settings
│
└── src/                       # Frontend Presentation Layer
    ├── core/                  # Shared Business Logic
    │   ├── store.ts           # State management
    │   └── events.ts          # Input intent definitions
    ├── components/            # Atomic Visual Elements
    │   ├── Button.tsx
    │   └── Modal.tsx
    ├── layouts/               # Form-Factor Specific Canvases
    │   ├── DesktopCanvas.tsx  # Fixed X/Y coordinates for Desktop
    │   └── MobileCanvas.tsx   # Fixed X/Y coordinates for Mobile
    ├── App.tsx                # Target profile router
    └── main.tsx               # Entry point & scale listener
```

---

## Summary Checklist for Implementation

1. **Configure Window Boundaries:** Lock aspect ratio and window dimensions in `tauri.conf.json`.
2. **Implement View Dispatcher:** Route the user to `DesktopCanvas` or `MobileCanvas` based on platform runtime checks.
3. **Setup Scaling Bridge:** Attach standard resize listeners to adjust `--scale-factor` on the root container.
4. **Abstract Inputs:** Map all mouse, keyboard, and touch interactions into unified intent handlers connected to Rust commands.