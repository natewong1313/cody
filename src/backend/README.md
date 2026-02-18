# backend
The backend is comprised of a TARPC server that is ran in its own thread by the sync engine.

### db
We use a local sqlite db file to store all data. This should be the ultimate source of truth for all application data.

### rpc
Rpc methods should generally be called in the gui directly only when calling mutations (create/update/delete). List/get methods should be called by the sync engine instead.

### harnesses
For now, we use opencode as the main harness but the backend is designed to eventually support changing harnesses in the future. We have a Harness trait that harnesses map their interface to. Any data that comes in or out should be translated into the higher level types in mod.rs.
