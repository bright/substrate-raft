Basic implementation of getting authority permission logic.

The current implementation relies on TiKV distributed KV database. It tries to
optimistically update slot/round/session number in case it's lower than the current one.
Success means the node is granted permission.
