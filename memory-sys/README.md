## Overview

This is the **lowest-level sub-library** of the project.  
Its purpose is to **perform inline assembly hooks** in Rust by creating **trampolines** that redirect function execution.  
It serves as the technical foundation for the higher-level OpenGL hooking and `egui` injection logic.

A **trampoline** in the context of detouring is a small code snippet that temporarily replaces the start of a target function with a jump to custom code, then eventually jumps back to the original function.  
Reference: [Low-Level Trampoline Hooking](https://en.wikipedia.org/wiki/Trampoline_(computing)).

This library is **experimental**, **unsafe**, and **highly prone to crashes** if the trampoline is not implemented correctly.

---

## Features

- **Manual function hooking** using `unsafe` Rust and inline assembly.
- **Low-level trampolines** for redirecting execution flow.
- Windows-specific debugging helpers:
  - `MessageBoxA` for in-game pop-up debug messages.
  - `OutputDebugStringA` for sending logs to any debugger attached to the process.
- Does **not** depend on any external hooking frameworks.

---

## Why Debugging is Different Here

When injecting a DLL into a game or other graphical application, you often **cannot use standard stdout or terminal logging** because:
- The target process is not running in a terminal.
- You don't have direct access to its console output.

Instead, you can:
- Use `OutputDebugStringA` to send logs to a debugger such as Visual Studio, x64dbg, or DebugView.
- Use `MessageBoxA` for quick, blocking visual debug prompts.

These methods are essential because they **do not rely on console access**.

---

## How It Works

1. **Injection Phase**  
   - The DLL is injected into the target process.

2. **Hook Installation**  
   - Inline assembly overwrites the beginning of a function with a jump to a custom handler.
   - The original bytes are copied into a **gateway/trampoline** buffer.
   - Control eventually jumps back to the original code after the hook logic.

3. **Execution Flow**  
   - Target process calls the hooked function.
   - Execution is redirected to our custom Rust function.
   - Debug output is optionally sent via `OutputDebugStringA` or `MessageBoxA`.
   - The original function flow resumes.

---

## Crash Risk

**Warning**: This is **extremely unsafe**.  
If the trampoline is misaligned, if instruction boundaries are not respected, or if memory protections are mishandled:
- The target process will **crash immediately**.
- In some cases, the crash may corrupt process state beyond recovery.

**Rule of thumb**: One wrong byte in the hook = instant crash.

---

## Potential Uses

- Learning how to build your own detour system from scratch.
- Creating debugging tools that hook internal functions of a target application.
- Studying Windows internals and function patching techniques.

---

## Project Status

- **Unmaintained**: No active development.
- **Reference only**: Intended for research and educational purposes.

---

## Disclaimer

This library is provided **as-is**.  
The author takes no responsibility for misuse, damage, or crashes caused by this code.  
Only use on software you own or have explicit permission to modify.

---

## License

MIT License — see [LICENSE](LICENSE) for details.