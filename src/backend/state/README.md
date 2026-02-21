# state

Helpers for managing in-memory state and subscriptions to that state.

The source of truth of data comes from the repos in the data folder. To make syncing performant, we maintain an in-memory cache of the repository data and we also manage subscriptions to a live feed of that data in here rather than at the repo level.
