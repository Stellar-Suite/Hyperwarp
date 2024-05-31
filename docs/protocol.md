# Fast data transfer network protocol
## Begin: Message Type (u32)
* 0 - MESSAGE
* 1 - hello
* 2 - my version is...
* 3 - DIE OUT OF CONFUSION

## Second: Message Length (u32 so max 4gb)

# Message Type specific protocol
## 2 - my version is
* string ig for comparison