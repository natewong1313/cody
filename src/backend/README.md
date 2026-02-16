# backend
The backend is comprised of an RPC server that is spawned by the gui on a seperate thread.

### harnesses
For now, we use opencode as the main harness but the backend is designed to eventually support changing harnesses in the future. We have a Harness trait that harnesses map their interface to. Any data that comes in or out should be translated into the higher level types in mod.rs
