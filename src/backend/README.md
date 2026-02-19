# backend
The backend is comprised of a TARPC server that is run in its own thread by live_query.

### db
We use a local sqlite db file to store all data. This should be the ultimate source of truth for all application data.

### rpc
live_query interacts with the backend via the TARPC server. In some instances you might need to directly call get_* methods to get data but the general flow should be preform mutations -> wait for data in backend to update and emit back to live_query. this is fine at the moment but we could add optimism down the line.

### harnesses
For now, we use opencode as the main harness but the backend is designed to eventually support changing harnesses in the future. We have a Harness trait that harnesses map their interface to. Any data that comes in or out should be translated into the higher level types in mod.rs.
