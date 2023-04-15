# SoundMod Native Design

## Goals
 * Heavy emphasis on the GPU
   * GPU-side Acoustic Path Tracing
   * Hybrid GPU/CPU UPS (uniformly partitioned) realtime audio convolution
 * Tight and least-invasive-possible integration with MC `SoundSystem`
 * Integration with other mods?
## Design
#### Crates
`native`: Implements core library functionality. This is where a majority of the logic will live and is a normal rust library.
`native-jni`: Depends on `native` and wraps its functionality in an FFI-safe interface. Built as a C dynamic library, packaged with the mod and linked to the JVM at runtime
`native-test`: Playground to test out the `native`s. Might go away in exchange for a `[[bin]]` target in `native`.

(the rest of this document will detail the design of the `native`) crate)
#### Modules
All of the modules in this project are designed to be "sandboxed" from each other and parallelized, with each module having a distinct thread/group of threads responsible for its logic.
* `interface`: This module handles all input and output to and from Minecraft. Still not sure if it should have its own thread or piggyback on the render thread to run its logic
  * `audio`: this submodule handles loading and decoding of MC audio resources into convenient sample buffers for use in convolution.
  * `world`: this submodule handles updating and managing material, location and world shape data, processing various Java callbacks into convenient state update messages which are passed on to the GPU path tracer
* `gpu`: this module contains all of the logic for creating and allocating buffers, shaders, and managing the GPU device state
  * `trace`: this submodule uses the tools defined in its parent to implement a GPU side acoustic path tracer
  * `convolve`: this submodule uses the tools defined in its parent to implement a hybrid CPU/GPU multi channel realtime convolution algorithm
* `playback`: this module handles all interfacing done with the underlying audio library (probably `cpal` or `rodio`) and final post processing. it is **imperative** that this thread blocks as little as possible to prevent stuttering
#### GPU path tracer structure
I'm planning on backing the path tracer with the venerable (and efficient) Amantides-Woo voxel grid raycasting algorithm.
 * World representation: The "world" gpu side will be represented in-memory as a buffer of u64s, where the presence of a block at a given position depends on the bit at an index in an array, where each u64 holds 64 array entries. The indexing scheme (and presence of run length encoding) are subject to change (I'm currently leaning towards chunks being self-sufficient and having each chunk have its own run-length-encoding, which would at least attentuate the large random access performance cost of a single, massive run length encoded buffer).
 * Materials: This is a tricky one. The naive implementation (and what I'll probably end up using for now) is just to index into a buffer of Material types with a u8, pack 4 of them into a u32, and store one byte per loaded block. Again, RLE could probably yield some big savings here.
#### Performance Musings
JNI calls can be expensive! C-based FFI is usually fairly cheap performance-wise in native languages. However, java isn't a native language, and it has to do a fair bit of work to setup a "native" calling convention and memory state, which can lead to expensive interrupts and context swaps. 
I'm considering backing the "hotter" callbacks (ie updating world mask state because a block was removed/placed) with a buffer that flushes (causing a native call) periodically or when filled.

