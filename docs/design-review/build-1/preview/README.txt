Collab Sheet static preview (D-UI-37).

Host this folder as it is on any static file server (it uses relative paths, so a sub-path works). No server
code, no external requests. Open index.html over http(s): browsers do not run module workers or fetch wasm from
file:// pages. For a quick local look: `npx vite preview --outDir .` or `python3 -m http.server` in this folder.

The page needs WebAssembly (CSP: script-src 'wasm-unsafe-eval'), a same-origin worker (worker-src 'self') and
BroadcastChannel. Collaboration is between the tabs of one browser; the document is saved in its localStorage.

Charts draw with the Carlito font (SIL Open Font License 1.1; licenses/Carlito-OFL.txt).
The icons are Quick's: Lucide icons (ISC License, Copyright (c) Lucide Contributors; licenses/lucide-ISC.txt) and
Quick's own drawings. icons/icons.svg is the set as an SVG sprite; icons/manifest.json says which is which.
