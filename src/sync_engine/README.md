# sync engine
Used by the client (gui) to have realtime data. Instead of having to deal with querying against the backend for data at start and managing the lifecycle, clients can instead subscribe to the data source and get all updates.


### use
In any page that needs data from the backend, make sure you call the ensure_*_loaded function before doing anything else inside the render loop.
```rust
page_ctx.sync_engine.ensure_projects_loaded();
```
Then, once inside a ui block call
```rust
page_ctx.sync_engine.poll(ui);
```

### rpc
The sync engine runs the backend rpc server in its own tokio thread.
