# Fast data transfer network protocol
## Begin: Message Type (u32)
* 0 - die out of confusion
* 1 - hello
* 2 - my version is...

## Second: Message Length (u32 so max 4gb)

# Message Type specific protocol
## 2 - my version is
* string ig for comparison