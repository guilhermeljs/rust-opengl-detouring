# OpenGL Detour + Overlay Injector (Main Project)

## Overview

This project hooks OpenGL 1.0 on Windows by locating the `OpenGL32.dll` and capturing a raw pointer to the function `wglSwapBuffers` using `GetProcAddress`.  
It creates a trampoline with the original instructions of `wglSwapBuffers` and builds a hook function that first calls our custom function, then calls the original function through the trampoline—effectively trampolining OpenGL.

Beyond hooking OpenGL, it manages OpenGL contexts by creating a **shared context**, allowing the game's original context to share resources with our overlay context.  
The overlay is rendered by hooking `wglSwapBuffers` to draw our OpenGL context on top of the game's frame.

This is extremely low-level code, using our internal unsafe library **`memory-sys`** to create trampolines and manipulate memory.

Additionally, the project implements a simple Lua function hook using the same trampoline method.

---

## Build

```sh
cargo build --release
```

---

## Usage

- Inject the compiled DLL into the target process using **ProcessHacker** or a similar injector.
- Since this is an injected DLL, there is **no standard console for logs**.
- To view logs, use **DebugView** or any tool that captures Windows **`OutputDebugString`** messages.

---

## Important Notes

- This project works at an **extremely low level**, relying heavily on unsafe Rust and manual memory patching.
- Incorrect trampoline creation or hook installation will crash the target process.
- Intended only for experimental and research purposes.

---

## Summary

- Locate `OpenGL32.dll` and hook `wglSwapBuffers` via trampoline.  
- Create a shared OpenGL context for overlay rendering.  
- Render overlay by hooking swap buffers.  
- Use internal unsafe library `memory-sys` for trampolines.  
- Supports a simple Lua trampoline hook.  
- Build with Cargo.  
- Inject with ProcessHacker.  
- View logs with DebugView.

---