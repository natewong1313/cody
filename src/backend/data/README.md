# data
Every data model should be backed by persistent storage (sqlite db currently) but some data models will also need to call into specific harnesses. The idea here is if a user swaps out a harness, we still have a record of the data.
